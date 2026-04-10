use std::marker::PhantomData;

use sqlx::{Database, Encode, database::HasStatementCache, prelude::Type};

use crate::{
    EncodeExtention, Expression, ExpressionToFragment, PositionalPlaceholder, QueryBuilder,
    SanitzingMechanisim,
};

pub struct DeferedCtx<S: Database> {
    pub ctx1: Vec<Option<Box<dyn Stored<S>>>>,
    pub output: S::Arguments<'static>,
}

pub struct DeferedFragment<S: Database>(pub Box<dyn FnOnce(&mut DeferedCtx<S>) -> String>);

impl<S: Database> DeferedFragment<S> {
    pub fn into_string(self, ctx: &mut DeferedCtx<S>) -> String {
        self.0(ctx)
    }
}

pub trait Stored<S>: 'static {
    fn bind_stored(self: Box<Self>, ctx: &mut S::Arguments<'static>)
    where
        S: Database;
}

impl<S, T> Stored<S> for T
where
    S: sqlx::Database,
    T: Type<S> + Encode<'static, S> + 'static + Send,
{
    #[inline]
    fn bind_stored(self: Box<Self>, ctx: &mut S::Arguments<'static>) {
        use sqlx::Arguments;
        ctx.add(*self).expect("internal bug, maybe?");
    }
}

pub struct defered_binder<S> {
    pub stored: Vec<Option<Box<dyn Stored<S>>>>,
    pub db: PhantomData<S>,
}

impl<S> Default for defered_binder<S> {
    fn default() -> Self {
        defered_binder {
            stored: Default::default(),
            db: PhantomData,
        }
    }
}

impl<S: Database> QueryBuilder for defered_binder<S> {
    type SqlxDb = S;
    type Fragment = DeferedFragment<S>;

    type Context = DeferedCtx<S>;

    type Output = S::Arguments<'static>;

    fn fragment_to_string(ctx: &mut Self::Context, fragment: Self::Fragment) -> String {
        fragment.into_string(ctx)
    }
    fn to_output(
        self,
        statement_builder: impl FnOnce(&mut Self::Context) -> String,
    ) -> (String, Self::Output) {
        let mut ctx = DeferedCtx {
            ctx1: self.stored,
            output: Default::default(),
        };
        let sql_statment = statement_builder(&mut ctx);
        return (sql_statment, ctx.output);
    }
}

impl<S, T> EncodeExtention<'static, T> for defered_binder<S>
where
    S: Database + PositionalPlaceholder,
    T: Stored<S>,
{
    fn encode(
        &mut self,
        val: T,
    ) -> impl FnOnce(&mut Self::Context) -> String + 'static + use<T, S> {
        self.stored.push(Some(Box::new(val)));
        let len = self.stored.len();
        move |ctx1| {
            let bring_back = ctx1
                .ctx1
                .get_mut(len - 1)
                .map(|e| e.take())
                .expect("item should be found")
                .expect(" and taken only once");
            bring_back.bind_stored(&mut ctx1.output);
            S::placeholder().to_string()
        }
    }
}

impl<S, E> ExpressionToFragment<'static, E> for defered_binder<S>
where
    E: Expression<'static, Self>,
    S: Database,
{
    fn expression_to_fragment(&mut self, t: E) -> <Self as QueryBuilder>::Fragment {
        let fnitem = t.expression(self);
        DeferedFragment(Box::new(fnitem))
    }
}

impl<S> SanitzingMechanisim for defered_binder<S>
where
    S: SanitzingMechanisim,
{
    type SanitzingMechanisim = S::SanitzingMechanisim;
}

#[cfg(test)]
mod tests {
    use sqlx::{
        Any as SqlxAny, Database,
        any::{AnyArguments, AnyTypeInfo},
    };
    use std::{marker::PhantomData, sync::Mutex};

    use sqlx::{
        Encode, Sqlite, Type,
        encode::IsNull,
        sqlite::{SqliteArgumentValue, SqliteArguments, SqliteTypeInfo, SqliteValue},
    };

    use crate::{
        Buildable,
        EncodeExtention,
        Expression,
        PositionalPlaceholder,
        QueryBuilder,
        SanitzingMechanisim,
        defered_builder::defered_binder,
        sanitize::{SanitizeAndHardcode, by_double_quote},
        statements::select_st::SelectSt, // prelude::{col, stmt::SelectSt},
    };

    struct StrButCountOrder(&'static str);

    static BIND_ORDER: Mutex<Vec<String>> = Mutex::new(Vec::new());

    impl<'q> Encode<'q, SqlxAny> for StrButCountOrder {
        fn encode_by_ref(
            &self,
            buf: &mut <SqlxAny as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx::error::BoxDynError> {
            panic!("should be called")
        }
        fn encode(
            self,
            buf: &mut <SqlxAny as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx::error::BoxDynError>
        where
            Self: Sized,
        {
            let mut s = BIND_ORDER.lock().unwrap();
            s.push(self.0.to_owned());
            drop(s);
            <String as Encode<'q, SqlxAny>>::encode(self.0.to_owned(), buf)
        }
    }

    impl<'q, Q> Expression<'q, Q> for StrButCountOrder
    where
        Q: QueryBuilder,
        Q::SqlxDb: Database,
        Q: EncodeExtention<'q, &'static str>,
    {
        fn expression(
            self,
            query_builder: &mut Q,
        ) -> impl FnOnce(&mut <Q>::Context) -> String + 'q + use<'q, Q>
        where
            Q: QueryBuilder,
        {
            let s = EncodeExtention::encode(query_builder, self.0);
            move |ctx| {
                let q = s(ctx);
                q
            }
        }
    }

    impl Type<SqlxAny> for StrButCountOrder {
        fn type_info() -> AnyTypeInfo {
            todo!()
        }
    }

    impl SanitzingMechanisim for SqlxAny {
        type SanitzingMechanisim = by_double_quote;
    }

    use crate::expressions::*;
    impl PositionalPlaceholder for SqlxAny {
        fn placeholder() -> &'static str {
            "?"
        }
    }

    #[test]
    fn positional_query_figure_out_order() {
        let mut st = SelectSt::init(
            "Todo",
            defered_binder {
                stored: Default::default(),
                db: PhantomData::<SqlxAny>,
            },
        );

        st.select(col("some_col"));
        st.where_(col("id").to_eq(StrButCountOrder("where")));
        st.offset(StrButCountOrder("offset"));

        let (str, arg) = st.build();

        drop(arg);

        assert_eq!(
            str,
            "SELECT 'some_col' FROM 'Todo' WHERE 'id' = ? OFFSET ?;"
        );

        let bind_order = BIND_ORDER.lock().unwrap().clone();

        assert_eq!(bind_order, vec!["where".to_string(), "offset".to_string()]);

        // even when we call offset before where,
        // PositionalQuery should know to reorder them
        BIND_ORDER.lock().unwrap().drain(..);
        let mut st = SelectSt::init(
            "Todo",
            defered_binder {
                stored: Default::default(),
                db: PhantomData::<SqlxAny>,
            },
        );

        st.select(col("some_col"));
        st.offset(StrButCountOrder("offset"));
        st.where_(col("id").to_eq(StrButCountOrder("where")));

        let (str, arg) = st.build();

        drop(arg);

        assert_eq!(
            str,
            "SELECT 'some_col' FROM 'Todo' WHERE 'id' = ? OFFSET ?;"
        );

        let bind_order = BIND_ORDER.lock().unwrap().clone();

        assert_eq!(bind_order, vec!["where".to_string(), "offset".to_string()]);
    }
}
