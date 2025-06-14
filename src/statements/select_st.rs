use std::marker::PhantomData;

use crate::{Accept, AcceptNoneBind, BindItem, Buildable, QueryBuilder, unstable::Unsateble};

pub struct SelectSt<S: QueryBuilder> {
    pub(crate) select_list: Vec<String>,
    pub(crate) where_clause: Vec<S::Fragment>,
    pub(crate) joins: Vec<Box<dyn Join>>,
    pub(crate) order_by: Vec<(String, bool)>,
    pub(crate) group_by: Option<String>,
    pub(crate) limit: Option<S::Fragment>,
    pub(crate) shift: Option<S::Fragment>,
    pub(crate) ctx: S::Context1,
    pub(crate) from: String,

    #[allow(unused)]
    pub(crate) ident_safety: (),
    pub(crate) _sqlx: PhantomData<S>,
}

pub trait Join: 'static + Send + Sync {
    fn display_from(self: Box<Self>, from: &str) -> String;
    fn global_table(&self) -> &str;
}

#[allow(non_camel_case_types)]
pub mod join {
    use super::Join;

    pub struct join {
        pub foriegn_table: String,
        pub foriegn_column: String,
        pub local_column: String,
    }

    impl Join for join {
        fn display_from(self: Box<Self>, from: &str) -> String {
            let Self {
                foriegn_table,
                foriegn_column,
                local_column,
            } = *self;
            format!(
                " JOIN {foriegn_table} ON {foriegn_table}.{foriegn_column} = {self_from}.{local_column}",
                self_from = from,
            )
        }
        fn global_table(&self) -> &str {
            self.foriegn_table.as_str()
        }
    }

    pub struct left_join {
        pub foriegn_table: String,
        pub foriegn_column: String,
        pub local_column: String,
    }
    impl Join for left_join {
        fn display_from(self: Box<Self>, from: &str) -> String {
            let Self {
                foriegn_table,
                foriegn_column,
                local_column,
            } = *self;
            format!(
                " LEFT JOIN {foriegn_table} ON {foriegn_table}.{foriegn_column} = {self_from}.{local_column}",
                self_from = from,
            )
        }

        fn global_table(&self) -> &str {
            self.foriegn_table.as_str()
        }
    }
}

pub mod order_by {
    pub const ASC: bool = true;
    pub const DESC: bool = false;
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
                str.push_str(item.as_ref());
            }

            str.push_str(" FROM ");
            str.push_str(self.from.as_ref());

            for join_ in self.joins.into_iter() {
                str.push_str(&join_.display_from(self.from.as_str()));
            }

            let mut where_str = Vec::default();
            for item in self.where_clause {
                let item = S::build_sql_part_back(ctx, item);
                if item.is_empty() {
                    continue;
                }

                where_str.push(item);
            }
            for (index, item) in where_str.into_iter().enumerate() {
                if index == 0 {
                    str.push_str(" WHERE ");
                } else {
                    str.push_str(" AND ");
                }
                str.push_str(&item);
            }

            if let Some(group_by) = self.group_by {
                str.push_str(" GROUP BY ");
                str.push_str(&group_by);
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
            group_by: None,
        }
    }

    pub fn group_by<T>(&mut self, item: T)
    where
        T: AcceptNoneBind<IdentSafety = ()>,
    {
        self.group_by = Some(item.accept(&self.ident_safety, Unsateble));
    }
    pub fn select<T>(&mut self, item: T)
    where
        T: AcceptNoneBind<IdentSafety = ()>,
    {
        self.select_list
            .push(item.accept(&self.ident_safety, Unsateble));
    }

    #[track_caller]
    pub fn join(&mut self, j: impl Join) {
        let global_table = j.global_table();
        if self
            .joins
            .iter()
            .find(|e| e.global_table() == global_table)
            .is_some()
        {
            panic!("table {} has been joined already", global_table);
        }

        self.joins.push(Box::new(j));
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
