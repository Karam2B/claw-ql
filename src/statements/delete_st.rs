use crate::{ BindItem, Buildable, QueryBuilder, expressions::ColEq, prelude::col};

use super::build_where;

pub struct DeleteSt<S: QueryBuilder> {
    pub(crate) where_clause: Vec<S::Fragment>,
    pub(crate) ctx: S::Context1,
    pub(crate) into_table: String,
    pub(crate) returning: Option<Vec<String>>,
}

impl<S: QueryBuilder> DeleteSt<S> {
    pub fn init(table_name: String) -> Self
    where
        ColEq<i64>: BindItem<S>,
    {
        let ctx = Default::default();
        Self {
            into_table: table_name,
            returning: Default::default(),
            where_clause: vec![],
            ctx,
        }
    }
    pub fn init_where_id_eq(table_name: String, id: i64) -> Self
    where
        ColEq<i64>: BindItem<S>,
    {
        let mut ctx = Default::default();
        Self {
            into_table: table_name,
            returning: Default::default(),
            where_clause: vec![S::handle_bind_item(col("id").eq(id), &mut ctx)],
            ctx,
        }
    }
}

impl<S: QueryBuilder> Buildable for DeleteSt<S> {
    type Database = S;

    #[track_caller]
    fn build(self) -> (String, S::Output) {
        S::build_query(self.ctx, |ctx| {
            let mut str = String::from("DELETE FROM ");

            str.push_str(self.into_table.as_ref());

            build_where::<S>(self.where_clause, ctx, &mut str);

            if let Some(returning) = self.returning {
                str.push_str(" RETURNING ");
                str.push_str(&returning.join(", "));
            }

            str.push(';');

            str
        })
    }
}

impl<S: QueryBuilder> DeleteSt<S> {
    pub fn where_<T>(&mut self, item: T)
    where
        T: BindItem<S> + 'static,
    {
        let item = S::handle_bind_item(item, &mut self.ctx);

        self.where_clause.push(item);
    }
    pub fn returning(mut self, cols: Vec<String>) -> Self {
        self.returning = Some(cols);
        self
    }
}
