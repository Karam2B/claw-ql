use crate::{Accept, BindItem, Buildable, QueryBuilder, expressions::ColEq, prelude::col};

pub struct UpdateOneSt<S: QueryBuilder> {
    pub(crate) sets: Vec<(String, S::Fragment)>,
    pub(crate) where_clause: Vec<S::Fragment>,
    pub(crate) ctx: S::Context1,
    pub(crate) into_table: String,
    pub(crate) returning: Option<Vec<String>>,
}

impl<S: QueryBuilder> UpdateOneSt<S> {
    pub fn init_where_id_eq(table_name: String, id: i64) -> Self
    where
        ColEq<i64>: BindItem<S>,
    {
        let mut ctx = Default::default();
        Self {
            into_table: table_name,
            returning: Default::default(),
            sets: Default::default(),
            where_clause: vec![S::handle_bind_item(col("id").eq(id), &mut ctx)],
            ctx,
        }
    }
    pub fn init(table_name: String) -> Self {
        Self {
            into_table: table_name,
            returning: Default::default(),
            ctx: Default::default(),
            sets: Default::default(),
            where_clause: Default::default(),
        }
    }
}

impl<S: QueryBuilder> Buildable for UpdateOneSt<S> {
    type Database = S;

    #[track_caller]
    fn build(self) -> (String, S::Output) {
        S::build_query(self.ctx, |ctx| {
            let mut str = String::from("UPDATE ");

            str.push_str(self.into_table.as_ref());

            str.push_str(" SET ");

            if self.sets.is_empty() {
                panic!("empty set on update")
            }

            for (index, (column, value)) in self.sets.into_iter().enumerate() {
                if index != 0 {
                    str.push_str(", ");
                }
                str.push_str(column.as_ref());
                str.push_str(" = ");
                str.push_str(&S::build_sql_part_back(ctx, value));
            }

            for (index, where_item) in self.where_clause.into_iter().enumerate() {
                if index == 0 {
                    str.push_str(" WHERE ");
                } else {
                    str.push_str(" AND ");
                }
                str.push_str(&S::build_sql_part_back(ctx, where_item));
            }

            if let Some(returning) = self.returning {
                str.push_str(" RETURNING ");
                str.push_str(&returning.join(", "));
            }

            str.push(';');

            str
        })
    }
}

impl<S: QueryBuilder> UpdateOneSt<S> {
    pub fn set_col<T>(&mut self, column: String, value: T)
    where
        S: Accept<T>,
        T: Send + 'static,
    {
        let part = S::handle_accept(value, &mut self.ctx);
        self.sets.push((column, part));
    }

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
