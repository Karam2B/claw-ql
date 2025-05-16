use std::marker::PhantomData;

use crate::QueryBuilder;

pub struct SelectSt<S: QueryBuilder> {
    pub(crate) select_list: Vec<(Option<String>, String, Option<&'static str>)>,
    pub(crate) where_clause: Vec<S::Fragment>,
    pub(crate) joins: Vec<(String, join)>,
    pub(crate) order_by: Vec<(String, bool)>,
    pub(crate) limit: Option<S::Fragment>,
    pub(crate) shift: Option<S::Fragment>,
    pub(crate) ctx: S::Context1,
    pub(crate) from: String,
    pub(crate) _sqlx: PhantomData<S>,
}

#[allow(non_camel_case_types)]
pub struct join {
    pub on_table: String,
    pub on_column: String,
    pub local_column: String,
}

impl<S: QueryBuilder> SelectSt<S> {
    pub fn init<T: AsRef<str>>(from: T) -> Self {
        SelectSt {
            select_list: Default::default(),
            where_clause: Default::default(),
            joins: vec![],
            order_by: Default::default(),
            limit: Default::default(),
            shift: Default::default(),
            ctx: Default::default(),
            from: from.as_ref().to_string(),
            _sqlx: PhantomData,
        }
    }
    pub fn build(self) -> (String, S::Output) {
        S::build_query(self.ctx, |ctx| {
            let mut str = String::from("SELECT ");

            if self.select_list.len() == 0 {
                panic!("select list is empty");
            }

            for (index, item) in self.select_list.into_iter().enumerate() {
                if index != 0 {
                    str.push_str(", ");
                }
                if let Some(table) = item.0 {
                    str.push_str(table.as_ref());
                    str.push_str(".");
                }
                str.push_str(item.1.as_ref());
                if let Some(alias) = item.2 {
                    str.push_str(" AS ");
                    str.push_str(alias);
                }
            }

            str.push_str(" FROM ");
            str.push_str(self.from.as_ref());

            for join in self.joins.into_iter() {
                let join = format!(
                    " {} {} ON {}.{} = {}.{}",
                    join.0,
                    join.1.on_table,
                    join.1.on_table,
                    join.1.on_column,
                    self.from,
                    join.1.local_column,
                );
                str.push_str(&join);
            }

            for (index, item) in self.where_clause.into_iter().enumerate() {
                let item = S::build_sql_part_back(ctx, item);
                if item.is_empty() {
                    // tracing::error!("item should not be empty {}", item);
                    continue;
                }
                if index == 0 {
                    str.push_str(" WHERE ");
                } else {
                    str.push_str(" AND ");
                }

                str.push_str(&item);
            }

            if self.order_by.len() != 0 {
                str.push_str(" ORDER BY ");
                for (index, (by, asc)) in self.order_by.into_iter().enumerate() {
                    if index != 0 {
                        str.push_str(", ");
                    }
                    str.push_str(by.as_ref());
                    if !asc {
                        str.push_str(" DESC");
                    }
                }
            }

            if let Some(limit) = self.limit {
                let limit = S::build_sql_part_back(ctx, limit);
                str.push_str(" LIMIT ");
                str.push_str(&limit);
            }

            if let Some(shift) = self.shift {
                let shift = S::build_sql_part_back(ctx, shift);
                str.push_str(" OFFSET ");
                str.push_str(&shift);
            }

            str.push_str(";");
            str
        })
    }
}
