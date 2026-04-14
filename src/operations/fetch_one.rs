use crate::collections::{Collection, Id, SingleIncremintalInt};
use crate::database_extention::DatabaseExt;
use crate::expressions::ColAs;
use crate::extentions::Members;
use crate::from_row::{FromRowAlias, pre_alias, two_alias};
use crate::links::Link;
use crate::operations::fetch_one::base_member_mod::BaseMember;
use crate::operations::{LinkedOutput, Operation};
use crate::query_builder::functional_expr::ManyFlat;
use crate::query_builder::{ManyExpressions, StatementBuilder};
use crate::statements::select_statement::SelectStatement;
use crate::use_executor;
use sqlx::{ColumnIndex, Database, Decode, Type};
use sqlx::{Executor, Row};

pub struct FetchOne<From, Links, Wheres> {
    pub base: From,
    pub wheres: Wheres,
    pub links: Links,
}

mod base_member_mod {
    use crate::database_extention::DatabaseExt;
    use crate::query_builder::Expression;
    use crate::query_builder::OpExpression;
    use crate::query_builder::StatementBuilder;

    pub struct BaseMember<T, C> {
        pub table: T,
        pub column: C,
    }
    impl<T, C> OpExpression for BaseMember<T, C> {}
    impl<'q, S, T, C> Expression<'q, S> for BaseMember<T, C>
    where
        S: DatabaseExt,
        T: AsRef<str> + 'q,
        C: AsRef<str> + 'q,
    {
        fn expression(self, arg: &mut StatementBuilder<'q, S>) {
            arg.sanitize(self.table.as_ref());
            arg.syntax(&".");
            arg.sanitize(self.column.as_ref());
            arg.syntax(&" AS ");
            let as_ = format!("b{}", self.column.as_ref());
            arg.sanitize(as_.as_str());
        }
    }
}

impl<S: Database, Base, Links, Wheres> Operation<S> for FetchOne<Base, Links, Wheres>
where
    S: DatabaseExt,
    for<'q> &'q str: ColumnIndex<S::Row>,
    Base: Collection<Data: Send, Id = SingleIncremintalInt> + 'static,
    <Base::Id as Id>::Data: for<'q> Decode<'q, S> + Type<S>,
    Base: Members,
    Wheres: ManyExpressions<'static, S>,
    Base: for<'r> FromRowAlias<'r, S::Row, FromRowData = Base::Data>,
    for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
    Links: Link<Base>,
    Links::Spec: Send
        + LinkFetchOne<
            S,
            Output: Send,
            Inner: Send,
            Joins: Send + ManyExpressions<'static, S>,
            Wheres: Send + ManyExpressions<'static, S>,
        >,
    Wheres: Send,
    Base: Send,
    Links: Send,
{
    type Output = Option<
        LinkedOutput<
            <Base::Id as Id>::Data,
            Base::Data,
            <<Links as Link<Base>>::Spec as LinkFetchOne<S>>::Output,
        >,
    >;

    async fn exec_operation(self, pool: &mut S::Connection) -> Self::Output
    where
        S: Database,
    {
        let link_spec = self.links.spec(&self.base);
        let link_extend_stmt = link_spec.non_aggregating_select_items();
        let link_extend_joins = link_spec.non_duplicating_joins();
        let link_extend_wheres = link_spec.wheres();

        let query_builder = StatementBuilder::<'_, S>::new(SelectStatement {
            select_items: ManyFlat((
                ColAs {
                    table: self.base.table_name().to_string(),
                    column: "id",
                    _as: "local_id",
                },
                self.base
                    .members_names()
                    .into_iter()
                    .map(|e| BaseMember {
                        table: self.base.table_name().to_string(),
                        column: e,
                    })
                    .collect::<Vec<_>>(),
                link_extend_stmt,
            )),
            from: self.base.table_name().to_string(),
            joins: link_extend_joins,
            wheres: ManyFlat((
                //
                link_extend_wheres,
                self.wheres,
            )),
            group_by: (),
            order: (),
            limit: (),
        });

        let stmt = query_builder.stmt().to_string();

        let s = use_executor!(fetch_optional(&mut *pool, query_builder));

        let s = match s {
            Err(sqlx::Error::RowNotFound) | Ok(None) => return None,
            Err(err) => {
                panic!(
                    "bug: claw_ql must clear all sqlx's error, 
it is hard to know where this error was originated!
error: {:?}
stmt: {stmt}",
                    err
                )
            }
            Ok(Some(ok)) => ok,
        };

        let (sub_op, inner) = link_spec.sub_op(two_alias(&s, "l", None, ()));

        let sub_op = sub_op.exec_operation(pool).await;

        Some(LinkedOutput {
            id: Row::get(&s, "local_id"),
            attributes: self
                .base
                .pre_alias(pre_alias(&s, "b"))
                .expect("bug: sqlx errors should ruled out by claw_ql"),
            links: link_spec.take(sub_op, inner),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct link_select_item<Table, Column> {
    pub table: Table,
    pub column: Column,
    /// only used by Vec<T> and tuples
    pub(crate) num: Option<usize>,
}

mod link_select_item_impls {
    use crate::database_extention::DatabaseExt;
    use crate::expressions::scoped_column;
    use crate::operations::fetch_one::link_select_item;
    use crate::query_builder::Expression;
    use crate::query_builder::OpExpression;
    use crate::query_builder::StatementBuilder;

    impl<Table, Column> link_select_item<Table, Column> {
        pub fn new_scoped(scoped_column: scoped_column<Table, Column>) -> Self {
            Self {
                table: scoped_column.table,
                column: scoped_column.column,
                num: None,
            }
        }
        pub fn new(table_: Table, column: Column) -> Self {
            Self {
                table: table_,
                column,
                num: None,
            }
        }
    }

    impl<T, C> OpExpression for link_select_item<T, C> {}

    impl<'q, S, T, C> Expression<'q, S> for link_select_item<T, C>
    where
        Self: 'q,
        S: DatabaseExt,
        T: AsRef<str> + 'q,
        C: AsRef<str> + 'q,
    {
        fn expression(self, arg: &mut StatementBuilder<'q, S>) {
            let alias = format!(
                "l{}{}",
                self.num.map(|num| num.to_string()).unwrap_or_default(),
                self.column.as_ref(),
            );

            arg.sanitize(self.table.as_ref());
            arg.syntax(&".");
            arg.sanitize(self.column.as_ref());
            arg.syntax(&" AS ");
            arg.sanitize(alias.as_str());
        }
    }
}

/// you can assume that all collections have an alias "local_id" of type `SingleIncremintalInt`
pub trait LinkFetchOne<S> {
    type Joins;
    type Wheres;

    /// select items has to be non aggregating in order to be extendable.
    ///
    /// example of aggregating select items is COUNT(*).
    ///
    /// use FetchAggregate instead.
    fn non_aggregating_select_items(&self) -> Vec<link_select_item<String, String>>;
    /// joins has to be non duplicating in order to be extendable
    ///
    /// example of duplicating joins is one-to-many relationship executed via a RIGHT JOIN on "many"-side table.
    ///
    /// to support duplicating joins, `FetchOne` needs to be refactored, which I think unnecessary, use sup_op instead.
    fn non_duplicating_joins(&self) -> Self::Joins;
    fn wheres(&self) -> Self::Wheres;

    type Inner;
    type SubOp: Operation<S>;
    fn sub_op(&self, row: two_alias<'_, <S as Database>::Row>) -> (Self::SubOp, Self::Inner)
    where
        S: Database;

    type Output;
    fn take(
        self,
        extend: <Self::SubOp as Operation<S>>::Output,
        inner: Self::Inner,
    ) -> Self::Output;
}

mod impl_for_tuples {
    use super::*;
    use crate::from_row::two_alias;
    impl<S> LinkFetchOne<S> for ()
    where
        S: Database,
    {
        type Joins = ();
        type Wheres = ();
        fn non_aggregating_select_items(&self) -> Vec<link_select_item<String, String>> {
            vec![]
        }
        fn non_duplicating_joins(&self) -> Self::Joins {
            ()
        }
        fn wheres(&self) -> Self::Wheres {
            ()
        }
        type Inner = ();
        type SubOp = ();
        fn sub_op(&self, _: two_alias<'_, <S as Database>::Row>) -> (Self::SubOp, Self::Inner) {
            ((), ())
        }
        type Output = ();
        fn take(self, _: <Self::SubOp as Operation<S>>::Output, _: Self::Inner) -> Self::Output {
            ()
        }
    }
}

#[cfg(test)]
mod test {
    use sqlx::{Executor, Sqlite};

    use crate::{
        connect_in_memory::ConnectInMemory,
        execute::Executable,
        expressions::{col_eq, scoped_column},
        operations::Operation,
        operations::{CollectionOutput, LinkedOutput},
        prelude::sql::FetchOne,
        test_module::*,
    };

    #[tokio::test]
    async fn test_fetch_one() {
        let pool = Sqlite::connect_in_memory().await;
        let mut tx = pool.begin().await.unwrap();

        Executor::execute(
            &pool,
            Executable {
                string: "
                CREATE TABLE IF NOT EXISTS Category (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS Todo (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    done BOOLEAN NOT NULL,
                    description TEXT,
                    category_id INTEGER,
                    FOREIGN KEY (category_id) REFERENCES Category(id)
                );

                INSERT INTO Category (id, title)
                    VALUES
                        (1, 'category_1');

                INSERT INTO Todo
                    (id, title, done, description, category_id)
                    VALUES
                        (1, 'first_todo', true, 'description_1', NULL),
                        (2, 'second_todo', false, NULL, 1);
                        ",
                arguments: Default::default(),
            },
        )
        .await
        .unwrap();

        let fetch_one = Operation::<Sqlite>::exec_operation(
            FetchOne {
                base: todo,
                wheres: col_eq {
                    col: scoped_column {
                        table: "Todo".to_string(),
                        column: "title".to_string(),
                    },
                    eq: "second_todo",
                },
                links: category,
            },
            tx.as_mut(),
        )
        .await;

        pretty_assertions::assert_eq!(
            fetch_one,
            Some(LinkedOutput {
                id: 2,
                attributes: Todo {
                    title: "second_todo".to_string(),
                    done: false,
                    description: None
                },
                links: Some(CollectionOutput {
                    id: 1,
                    attributes: Category {
                        title: "category_1".to_string()
                    }
                }),
            })
        );
    }
}

mod functional_impls {
    impl<S> LinkFetchOne<S> for ()
    where
        S: Database,
    {
        type Joins = ();
        type Wheres = ();
        fn non_aggregating_select_items(&self) -> Vec<link_select_item<String, String>> {
            vec![]
        }
        fn non_duplicating_joins(&self) -> Self::Joins {
            ()
        }
        fn wheres(&self) -> Self::Wheres {
            ()
        }
        type Inner = ();
        type SubOp = ();
        fn sub_op(&self, _: two_alias<'_, <S as Database>::Row>) -> (Self::SubOp, Self::Inner) {
            ((), ())
        }
        type Output = ();
        fn take(self, _: <Self::SubOp as Operation<S>>::Output, _: Self::Inner) -> Self::Output {
            ()
        }
    }
    impl<T, S> LinkFetchOne<S> for Vec<T>
    where
        T: LinkFetchOne<S>,
        T::Joins: IntoIterator,
        T::Wheres: IntoIterator,
    {
        type Joins = ManyFlat<Vec<<T::Joins as IntoIterator>::Item>>;

        type Wheres = ManyFlat<Vec<<T::Wheres as IntoIterator>::Item>>;

        fn non_aggregating_select_items(&self) -> Vec<link_select_item<String, String>> {
            let mut items = vec![];
            for (index, each) in self.iter().enumerate() {
                let mut each = each.non_aggregating_select_items();
                each.iter_mut().for_each(|e| e.num = Some(index));
                items.extend(each);
            }
            items
        }

        fn non_duplicating_joins(&self) -> Self::Joins {
            let mut joins = vec![];
            for each in self {
                joins.extend(each.non_duplicating_joins());
            }
            ManyFlat(joins)
        }

        fn wheres(&self) -> Self::Wheres {
            let mut wheres = vec![];
            for each in self {
                wheres.extend(each.wheres());
            }
            ManyFlat(wheres)
        }

        type Inner = Vec<T::Inner>;

        type SubOp = Vec<T::SubOp>;

        fn sub_op(
            &self,
            row: crate::from_row::two_alias<'_, <S as sqlx::Database>::Row>,
        ) -> (Self::SubOp, Self::Inner)
        where
            S: sqlx::Database,
        {
            let mut sup_op = vec![];
            let mut inner = vec![];

            if row.2.is_some() {
                panic!("vec<T> should not be nested")
            }

            for (index, each) in self.iter().enumerate() {
                let each = each.sub_op(two_alias(row.0, row.1, Some(index), ()));
                sup_op.push(each.0);
                inner.push(each.1);
            }

            (sup_op, inner)
        }

        type Output = Vec<T::Output>;

        fn take(
            self,
            extend: <Self::SubOp as crate::operations::Operation<S>>::Output,
            inner: Self::Inner,
        ) -> Self::Output {
            let mut this = self.into_iter();
            let mut extend = extend.into_iter();
            let mut inner = inner.into_iter();

            let mut out = vec![];

            loop {
                match (this.next(), extend.next(), inner.next()) {
                    (Some(this), Some(extend), Some(inner)) => {
                        out.push(this.take(extend, inner));
                    }
                    (None, None, None) => break,
                    _ => {
                        panic!("bug: unmatched lenght ")
                    }
                }
            }

            out
        }
    }
}
