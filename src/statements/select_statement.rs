use std::ops::Not;

use crate::{
    database_extention::DatabaseExt,
    query_builder::{
        Expression, OpExpression, PossibleExpression, QueryBuilder, ZeroOrMoreExpressions,
    },
};

pub struct SelectStatement<SelectItems, From, Joins, Wheres, Order, Limit> {
    pub select_items: SelectItems,
    pub from: From,
    pub joins: Joins,
    pub wheres: Wheres,
    pub order: Order,
    pub limit: Limit,
}

impl<SelectItems, From, Joins, Wheres, Limit, Order> OpExpression
    for SelectStatement<SelectItems, From, Joins, Wheres, Limit, Order>
{
}

impl<'q, S, SelectItems, From, Joins, Wheres, Limit, Order> Expression<'q, S>
    for SelectStatement<SelectItems, From, Joins, Wheres, Limit, Order>
where
    SelectItems: ZeroOrMoreExpressions<'q, S> + 'q,
    From: Expression<'q, S> + 'q,
    Joins: ZeroOrMoreExpressions<'q, S> + 'q,
    Wheres: ZeroOrMoreExpressions<'q, S> + 'q,
    Limit: PossibleExpression<'q, S> + 'q,
    Order: PossibleExpression<'q, S> + 'q,
{
    #[track_caller]
    fn expression(self, ctx: &mut QueryBuilder<'q, S>)
    where
        S: DatabaseExt,
    {
        ctx.syntax("SELECT ");
        if self.select_items.is_op().not() {
            panic!("empty select item")
        }
        self.select_items.expression("", ", ", ctx);

        ctx.syntax(" FROM ");
        self.from.expression(ctx);
        self.joins.expression(" ", " ", ctx);
        self.wheres.expression(" WHERE ", " AND ", ctx);
        self.order.expression_starting(" ORDER ", ctx);
        self.limit.expression_starting(" LIMIT ", ctx);
        ctx.syntax(";");
    }
}

#[cfg(feature = "skip_without_comments")]
// the old api, waiting for a second rewrite
pub mod old_dynamic_statement {
    use std::marker::PhantomData;

    use crate::{
        Buildable, ExpressionToFragment, JoinItem, QueryBuilder, SanitzingMechanisim,
        SelectListItem, WhereItem, sanitize::SanitizeAndHardcode,
    };

    pub struct SelectSt<S: QueryBuilder> {
        pub(crate) select_list: Vec<String>,
        pub(crate) where_clause: Vec<S::Fragment>,
        pub(crate) joins: Vec<String>,
        pub(crate) order_by: Vec<(String, bool)>,
        pub(crate) group_by: Option<String>,
        pub(crate) limit: Option<S::Fragment>,
        pub(crate) shift: Option<S::Fragment>,
        pub(crate) ctx: S,
        pub(crate) from: String,
        pub(crate) _sqlx: PhantomData<S>,
    }

    #[allow(non_camel_case_types)]
    pub mod join {
        use crate::{
            JoinItem,
            sanitize::{SanitizeAndHardcode, by_double_quote},
        };

        pub struct join {
            pub foriegn_table: String,
            pub foriegn_column: String,
            pub local_table: String,
            pub local_column: String,
        }

        impl JoinItem for join {}
        impl SanitizeAndHardcode<by_double_quote> for join {
            fn sanitize(&self) -> String {
                format!(
                    " JOIN {foriegn_table} ON {foriegn_table}.{foriegn_column} = {local_table}.{local_column}",
                    foriegn_table = String::sanitize(&self.foriegn_table),
                    foriegn_column = String::sanitize(&self.foriegn_column),
                    local_table = String::sanitize(&self.local_table),
                    local_column = String::sanitize(&self.local_column),
                )
            }
        }

        pub struct left_join {
            pub foriegn_table: String,
            pub foriegn_column: String,
            pub local_table: String,
            pub local_column: String,
        }

        impl JoinItem for left_join {}

        impl SanitizeAndHardcode<by_double_quote> for left_join {
            fn sanitize(&self) -> String {
                format!(
                    "LEFT JOIN {foriegn_table} ON {foriegn_table}.{foriegn_column} = {local_table}.{local_column}",
                    foriegn_table = String::sanitize(&self.foriegn_table),
                    foriegn_column = String::sanitize(&self.foriegn_column),
                    local_table = String::sanitize(&self.local_table),
                    local_column = String::sanitize(&self.local_column),
                )
            }
        }
    }

    pub mod order_by {
        pub const ASC: bool = true;
        pub const DESC: bool = false;
    }

    impl<S: QueryBuilder> Buildable for SelectSt<S> {
        type QueryBuilder = S;

        #[track_caller]
        fn build(self) -> (String, <S as QueryBuilder>::Output) {
            S::to_output(self.ctx, |ctx| {
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
                    str.push_str(&join_);
                }

                let mut where_str = Vec::default();
                for item in self.where_clause {
                    let item = S::fragment_to_string(ctx, item);
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
                    let limit = S::fragment_to_string(ctx, limit);
                    str.push_str(" LIMIT ");
                    str.push_str(&limit);
                }

                if let Some(shift) = self.shift {
                    let shift = S::fragment_to_string(ctx, shift);
                    str.push_str(" OFFSET ");
                    str.push_str(&shift);

                    // panic!("hit {:?}", shift);
                }

                str.push_str(";");
                str
            })
        }
    }

    impl<S: QueryBuilder + Default> SelectSt<S> {
        pub fn init<T>(from: T, query_builder: S) -> Self
        where
            S: SanitzingMechanisim,
            T: SelectListItem + SanitizeAndHardcode<S::SanitzingMechanisim>,
        {
            SelectSt {
                select_list: Default::default(),
                where_clause: Default::default(),
                joins: vec![],
                order_by: Default::default(),
                limit: Default::default(),
                shift: Default::default(),
                ctx: Default::default(),
                from: from.sanitize(),
                _sqlx: PhantomData,
                group_by: None,
            }
        }

        pub fn group_by<T>(&mut self, item: T)
        where
            S: SanitzingMechanisim,
            T: SanitizeAndHardcode<S::SanitzingMechanisim>,
        {
            self.group_by = Some(item.sanitize())
        }
        pub fn select<T>(&mut self, item: T)
        where
            S: SanitzingMechanisim,
            T: SelectListItem,
            T: SanitizeAndHardcode<S::SanitzingMechanisim>,
        {
            self.select_list.push(item.sanitize())
        }

        #[track_caller]
        pub fn join<T>(&mut self, j: T)
        where
            S: SanitzingMechanisim,
            T: JoinItem + SanitizeAndHardcode<S::SanitzingMechanisim>,
        {
            self.joins.push(j.sanitize());
        }

        pub fn order_by(&mut self, by: String, asc: bool) {
            self.order_by.push((by, asc));
        }

        #[track_caller]
        pub fn offset<'q, T>(&mut self, shift: T)
        where
            S: ExpressionToFragment<'q, T>,
            T: Send + 'q,
        {
            if self.shift.is_some() {
                panic!("limit has been set already");
            }

            let limit = S::expression_to_fragment(&mut self.ctx, shift);

            self.shift = Some(limit);
        }

        #[track_caller]
        pub fn limit<'q, T>(&mut self, limit: T)
        where
            S: ExpressionToFragment<'q, T>,
            T: Send + 'q,
        {
            if self.limit.is_some() {
                panic!("limit has been set already");
            }

            let limit = S::expression_to_fragment(&mut self.ctx, limit);

            self.limit = Some(limit);
        }
        pub fn where_<'q, T>(&mut self, item: T)
        where
            T: WhereItem<String>,
            S: ExpressionToFragment<'q, T>,
        {
            let item = S::expression_to_fragment(&mut self.ctx, item);
            self.where_clause.push(item);
        }
    }
}
