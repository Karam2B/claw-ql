use sqlx::{Any as SqlxAny, Database, Encode, prelude::Type};

use crate::{Accept, QueryBuilder};

pub struct DeferedFragment<S: Database>(pub Box<dyn FnOnce(&mut DeferedCtx<S>) -> String>);

impl<S: Database> DeferedFragment<S> {
    pub fn build_sql_fragment_back(self, ctx: &mut DeferedCtx<S>) -> String {
        self.0(ctx)
    }
}

pub trait Stored<S> {
    fn bind(self: Box<Self>, ctx: &mut S::Arguments<'static>)
    where
        S: Database;
}

impl<S, T> Stored<S> for T
where
    S: Database,
    T: Type<S> + for<'q> Encode<'q, S> + 'static + Send,
{
    #[inline]
    fn bind(self: Box<Self>, ctx: &mut <S>::Arguments<'static>)
    where
        S: Database,
    {
        use sqlx::Arguments;
        ctx.add(*self).expect("internal bug, maybe?");
    }
}

pub struct DeferedCtx<S: Database> {
    ctx1: Vec<Option<Box<dyn Stored<S>>>>,
    output: S::Arguments<'static>,
}

impl<S: Database> From<Vec<Option<Box<dyn Stored<S>>>>> for DeferedCtx<S> {
    fn from(ctx1: Vec<Option<Box<dyn Stored<S>>>>) -> Self {
        Self {
            ctx1,
            output: Default::default(),
        }
    }
}

impl QueryBuilder for SqlxAny {
    type Fragment = DeferedFragment<SqlxAny>;

    type Context1 = Vec<Option<Box<dyn Stored<SqlxAny>>>>;

    type Context2 = DeferedCtx<SqlxAny>;

    fn build_sql_part_back(ctx: &mut Self::Context2, from: Self::Fragment) -> String {
        from.build_sql_fragment_back(ctx)
    }

    type Output = <SqlxAny as Database>::Arguments<'static>;

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        let mut ctx2: DeferedCtx<SqlxAny> = ctx1.into();
        let str = f(&mut ctx2);
        return (str, ctx2.output);
    }

    fn handle_bind_item<T>(t: T, ctx: &mut Self::Context1) -> Self::Fragment
    where
        T: super::BindItem<Self> + 'static,
    {
        DeferedFragment(Box::new(move |ctx| {
            let bring_back = t.bind_item(&mut ctx.ctx1);
            bring_back(ctx)
        }))
    }

    fn handle_accept<T>(t: T, ctx: &mut Self::Context1) -> Self::Fragment
    where
        T: 'static + Send,
        Self: Accept<T>,
    {
        DeferedFragment(Box::new(move |ctx| {
            let bring_back = <Self as Accept<T>>::accept(t, &mut ctx.ctx1);
            bring_back(ctx)
        }))
    }
}

impl<T> Accept<T> for SqlxAny
where
    T: Type<SqlxAny> + for<'q> Encode<'q, SqlxAny> + 'static + Send,
{
    fn accept(
        this: T,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send + use<T> {
        ctx1.push(Some(Box::new(this)));
        let len = ctx1.len();
        move |ctx1| {
            let bring_back = ctx1
                .ctx1
                .get_mut(len - 1)
                .map(|e| e.take())
                // two options nested!
                .flatten()
                .expect("should be taken only once");
            bring_back.bind(&mut ctx1.output);
            "?".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{
        Any as SqlxAny, Database,
        any::{AnyArguments, AnyTypeInfo},
    };
    use std::sync::Mutex;

    use sqlx::{
        Encode, Sqlite, Type,
        encode::IsNull,
        sqlite::{SqliteArgumentValue, SqliteArguments, SqliteTypeInfo, SqliteValue},
    };

    use crate::{
        Buildable,
        prelude::{col, stmt::SelectSt},
    };

    struct StrButCountOrder(&'static str);

    static BIND_ORDER: Mutex<Vec<String>> = Mutex::new(Vec::new());

    impl<'q> Encode<'q, SqlxAny> for StrButCountOrder {
        fn encode_by_ref(
            &self,
            buf: &mut <SqlxAny as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx::error::BoxDynError> {
            BIND_ORDER.lock().unwrap().push(self.0.to_owned());
            <String as Encode<'q, SqlxAny>>::encode_by_ref(&self.0.to_owned(), buf)
        }
        fn encode(
            self,
            buf: &mut <SqlxAny as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx::error::BoxDynError>
        where
            Self: Sized,
        {
            BIND_ORDER.lock().unwrap().push(self.0.to_owned());
            <String as Encode<'q, SqlxAny>>::encode(self.0.to_owned(), buf)
        }
    }

    impl Type<SqlxAny> for StrButCountOrder {
        fn type_info() -> AnyTypeInfo {
            todo!()
        }
    }

    #[test]
    fn positional_query_figure_out_order() {
        let mut st = SelectSt::<SqlxAny>::init("Todo");

        st.select(col("*"));
        st.where_(col("id").eq(StrButCountOrder("1")));
        st.offset(StrButCountOrder("2"));

        let (str, arg) = st.build();

        drop(arg);

        assert_eq!(str, "SELECT * FROM Todo WHERE id = ? OFFSET ?;");

        let bind_order = BIND_ORDER.lock().unwrap().clone();

        assert_eq!(bind_order, vec!["1".to_string(), "2".to_string()]);


        // even when we call offset before where,
        // PositionalQuery should know to reorder them
        BIND_ORDER.lock().unwrap().drain(..);
        let mut st = SelectSt::<SqlxAny>::init("Todo");

        st.select(col("*"));
        st.offset(StrButCountOrder("2"));
        st.where_(col("id").eq(StrButCountOrder("1")));

        let (str, arg) = st.build();

        drop(arg);

        assert_eq!(str, "SELECT * FROM Todo WHERE id = ? OFFSET ?;");

        let bind_order = BIND_ORDER.lock().unwrap().clone();

        assert_eq!(bind_order, vec!["1".to_string(), "2".to_string()]);
    }
}
