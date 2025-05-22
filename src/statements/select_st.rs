use std::marker::PhantomData;

use crate::{Accept, AcceptNoneBind, BindItem, Buildable, QueryBuilder, unstable::Unsateble};

pub struct SelectSt<S: QueryBuilder> {
    pub(crate) select_list: Vec<(Option<String>, String, Option<&'static str>)>,
    pub(crate) where_clause: Vec<S::Fragment>,
    pub(crate) joins: Vec<join>,
    pub(crate) order_by: Vec<(String, bool)>,
    pub(crate) limit: Option<S::Fragment>,
    pub(crate) shift: Option<S::Fragment>,
    pub(crate) ctx: S::Context1,
    pub(crate) from: String,

    #[allow(unused)]
    pub(crate) ident_safety: (),
    pub(crate) _sqlx: PhantomData<S>,
}

#[allow(non_camel_case_types)]
pub enum join {
    left_join {
        foriegn_table: String,
        foriegn_column: String,
        local_column: String,
    },
}

pub mod order_by {
    pub const ASC: bool = true;
    pub const DESC: bool = false;
}

impl join {
    pub fn global_table(&self) -> &str {
        match self {
            Self::left_join { foriegn_table, .. } => foriegn_table.as_str(),
        }
    }
}

impl<S: QueryBuilder> Buildable for SelectSt<S> {
    type Database = S;

    #[track_caller]
    fn build(self) -> (String, <S as QueryBuilder>::Output) {
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

            for join_ in self.joins.into_iter() {
                match join_ {
                    join::left_join {
                        foriegn_table,
                        foriegn_column,
                        local_column,
                    } => {
                        let join = format!(
                            " LEFT JOIN ON {}.{} = {}.{}",
                            foriegn_table, foriegn_column, self.from, local_column,
                        );
                        str.push_str(&join);
                    }
                }
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
            ident_safety: (),
        }
    }

    pub fn select<T>(&mut self, item: T)
    where
        T: AcceptNoneBind<IdentSafety = ()>,
    {
        self.select_list.push((None, item.accept(Unsateble), None));
    }

    #[track_caller]
    pub fn join(&mut self, j: join) {
        if self
            .joins
            .iter()
            .find(|e| e.global_table() == j.global_table())
            .is_some()
        {
            panic!("table {} has been joined already", j.global_table());
        }

        self.joins.push(j);
    }

    pub fn order_by(&mut self, by: String, asc: bool) {
        self.order_by.push((by, asc));
    }

    #[track_caller]
    pub fn offset<T>(&mut self, shift: T)
    where
        S: Accept<T>,
        T: Send + 'static,
    {
        if self.shift.is_some() {
            panic!("limit has been set already");
        }

        let limit = S::handle_accept(shift, &mut self.ctx);

        self.shift = Some(limit);
    }

    #[track_caller]
    pub fn limit<T>(&mut self, limit: T)
    where
        S: Accept<T>,
        T: Send + 'static,
    {
        if self.limit.is_some() {
            panic!("limit has been set already");
        }

        let limit = S::handle_accept(limit, &mut self.ctx);

        self.limit = Some(limit);
    }
    pub fn where_<T>(&mut self, item: T)
    where
        T: BindItem<S> + 'static,
    {
        let item = S::handle_bind_item(item, &mut self.ctx);

        self.where_clause.push(item);
    }
}
