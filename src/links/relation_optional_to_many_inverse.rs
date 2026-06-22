use crate::links::{LinkedToBase, LinkedViaIds};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct OptionalToManyInverse<Id, F, T> {
    pub fk_unique_id: Id,
    pub from: F,
    pub to: T,
}

impl<Id, F, T> LinkedViaIds for OptionalToManyInverse<Id, F, T> {}

impl<Id, F, T> LinkedToBase for OptionalToManyInverse<Id, F, T> {
    type Base = F;
}

mod post_op {
    use std::collections::HashMap;

    use sqlx::{Decode, Encode, Row, Type};

    use crate::{
        collections::{Collection, CollectionId, SingleColumnId},
        database_extention::DatabaseExt,
        execute::Executable,
        expressions::table,
        extentions::{
            Members,
            common_expressions::{Identifier, TableNameExpression},
        },
        fix_executor::ExecutorTrait,
        from_row::FromRowAlias,
        links::relation_optional_to_many_inverse::OptionalToManyInverse,
        operations::{CollectionOutput, Operation, OperationOutput},
        query_builder::{Expression, OpExpression, StatementBuilder},
        statements::select_statement::SelectStatement,
    };

    pub type OptionalToManyInverseLinkedMap<FromId, ToId, ToOutput> =
        HashMap<FromId, Vec<CollectionOutput<ToId, ToOutput>>>;

    #[derive(Clone)]
    pub struct ColumnIn<Col, V> {
        pub col: Col,
        pub values: Vec<V>,
    }

    impl<Col, V> OpExpression for ColumnIn<Col, V> {}

    impl<'q, S, Col, V> Expression<'q, S> for ColumnIn<Col, V>
    where
        S: DatabaseExt,
        Col: Expression<'q, S> + 'q,
        V: 'q + Encode<'q, S> + Type<S> + Clone,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            self.col.expression(ctx);
            ctx.syntax(" IN (");
            for (i, value) in self.values.into_iter().enumerate() {
                if i > 0 {
                    ctx.syntax(", ");
                }
                ctx.bind(value);
            }
            ctx.syntax(")");
        }
    }

    struct PostOpSelect {
        fk_col: String,
        to_table: String,
        to_cols: Vec<String>,
    }

    impl OpExpression for PostOpSelect {}

    impl<'q, S> Expression<'q, S> for PostOpSelect
    where
        S: DatabaseExt,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.sanitize(&self.to_table);
            ctx.syntax(".");
            ctx.sanitize(&self.fk_col);
            ctx.syntax(r#" AS "from_id", "#);
            ctx.sanitize(&self.to_table);
            ctx.syntax(".");
            ctx.sanitize("id");
            for col in &self.to_cols {
                ctx.syntax(", ");
                ctx.sanitize(&self.to_table);
                ctx.syntax(".");
                ctx.sanitize(col);
            }
        }
    }

    pub struct FetchOptionalToManyInverseLinked<Key, From, To>
    where
        From: Collection,
        To: Collection + Members + TableNameExpression,
    {
        pub link: OptionalToManyInverse<Key, From, To>,
        pub from_ids: Vec<<From::Id as CollectionId>::IdData>,
        fk_col: String,
        to_table: String,
        to_cols: Vec<String>,
    }

    impl<Key, From, To> Clone for FetchOptionalToManyInverseLinked<Key, From, To>
    where
        Key: Clone,
        From: Collection + Clone,
        To: Collection + Members + TableNameExpression + Clone,
        <From::Id as CollectionId>::IdData: Clone,
    {
        fn clone(&self) -> Self {
            Self {
                link: self.link.clone(),
                from_ids: self.from_ids.clone(),
                fk_col: self.fk_col.clone(),
                to_table: self.to_table.clone(),
                to_cols: self.to_cols.clone(),
            }
        }
    }

    impl<Key, From, To> OperationOutput for FetchOptionalToManyInverseLinked<Key, From, To>
    where
        From: Collection,
        To: Collection + Members + TableNameExpression,
    {
        type Output = OptionalToManyInverseLinkedMap<
            <From::Id as CollectionId>::IdData,
            <To::Id as CollectionId>::IdData,
            To::OutputData,
        >;
    }

    impl<Key, From, To> FetchOptionalToManyInverseLinked<Key, From, To>
    where
        Key: Clone + AsRef<str>,
        From: Collection + TableNameExpression + Clone,
        To: Collection + TableNameExpression + Members + Clone,
    {
        pub fn new(
            link: OptionalToManyInverse<Key, From, To>,
            from_ids: Vec<<From::Id as CollectionId>::IdData>,
        ) -> Self {
            let to = link.to.clone();
            Self {
                fk_col: format!(
                    "fk_{}{}",
                    link.from.table_name_lower_case(),
                    link.fk_unique_id.as_ref(),
                ),
                to_table: to.table_name().to_string(),
                to_cols: to.members_names(),
                link,
                from_ids,
            }
        }
    }

    impl<S, Key, From, To> Operation<S> for FetchOptionalToManyInverseLinked<Key, From, To>
    where
        S: DatabaseExt + ExecutorTrait,
        Key: Clone + AsRef<str> + Send,
        From: Collection<Id: SingleColumnId> + TableNameExpression + Clone + Send,
        <From::Id as CollectionId>::IdData:
            Copy + Clone + std::hash::Hash + Eq + Send + for<'q> Encode<'q, S> + Type<S> + for<'r> Decode<'r, S>,
        To: Collection<Id: SingleColumnId + Identifier> + TableNameExpression + Members + Clone + Send,
        To::OutputData: Send,
        <To::Id as CollectionId>::IdData: Send,
        To: for<'r> FromRowAlias<'r, S::Row, RData = To::OutputData>,
        To::Id: for<'r> FromRowAlias<'r, S::Row, RData = <To::Id as CollectionId>::IdData>,
        <To::Id as Identifier>::Identifier: for<'q> Expression<'q, S>,
        for<'a> &'a str: sqlx::ColumnIndex<S::Row>,
    {
        async fn exec_operation(self, pool: &mut S::Connection) -> Self::Output {
            if self.from_ids.is_empty() {
                return HashMap::new();
            }

            let to_table = self.to_table.clone();
            let fk_col = self.fk_col.clone();

            let (stmt, args) = StatementBuilder::<'_, S>::new(SelectStatement {
                select_items: PostOpSelect {
                    fk_col: fk_col.clone(),
                    to_table: to_table.clone(),
                    to_cols: self.to_cols,
                },
                from: table(to_table.clone()),
                joins: (),
                wheres: ColumnIn {
                    col: table(to_table).col(fk_col),
                    values: self.from_ids,
                },
                group_by: (),
                order: (),
                limit: (),
            })
            .unwrap();

            let rows = S::fetch_all(
                &mut *pool,
                Executable {
                    string: &stmt,
                    arguments: args,
                },
            )
            .await
            .unwrap();

            let to = self.link.to.clone();
            let to_id = to.id();
            let mut map = HashMap::new();

            for row in rows {
                let from_id = row
                    .try_get::<<From::Id as CollectionId>::IdData, _>("from_id")
                    .unwrap();
                let id = to_id.no_alias(&row).unwrap();
                let attributes = to.no_alias(&row).unwrap();
                map.entry(from_id)
                    .or_insert_with(Vec::new)
                    .push(CollectionOutput { id, attributes });
            }

            map
        }
    }
}

mod impl_link_fetch {
    use std::collections::HashSet;

    use crate::{
        collections::{Collection, CollectionId, SingleColumnId},
        extentions::{Members, common_expressions::{Aliased, TableNameExpression}},
        from_row::FromRowData,
        links::relation_optional_to_many_inverse::{
            OptionalToManyInverse, post_op::FetchOptionalToManyInverseLinked,
        },
        links::relation_optional_to_many_inverse::post_op::OptionalToManyInverseLinkedMap,
        operations::{CollectionOutput, ManyLinkOutput, OperationOutput, fetch_many::LinkFetch},
    };

    impl<Key, From, To> LinkFetch for OptionalToManyInverse<Key, From, To>
    where
        Key: Clone + AsRef<str>,
        From: Collection<Id: SingleColumnId + Aliased> + TableNameExpression + Clone,
        To: Collection<Id: SingleColumnId> + TableNameExpression + Members + Clone,
        <From::Id as CollectionId>::IdData: Copy + Clone + std::hash::Hash + Eq,
        From::Id: FromRowData<RData = <From::Id as CollectionId>::IdData>,
        FetchOptionalToManyInverseLinked<Key, From, To>: OperationOutput<
            Output = OptionalToManyInverseLinkedMap<
                <From::Id as CollectionId>::IdData,
                <To::Id as CollectionId>::IdData,
                To::OutputData,
            >,
        >,
    {
        type SelectItems = From::Id;

        fn non_aggregating_select_items(&self) -> Self::SelectItems {
            self.from.id()
        }

        type Join = ();

        fn non_duplicating_join_expressions(&self) -> Self::Join {}

        type Wheres = ();

        fn where_expressions(&self) -> Self::Wheres {}

        type Op = FetchOptionalToManyInverseLinked<Key, From, To>;

        type Output =
            ManyLinkOutput<CollectionOutput<<To::Id as CollectionId>::IdData, To::OutputData>>;

        fn take_many(
            &self,
            from_id: <Self::SelectItems as FromRowData>::RData,
            op: &mut <Self::Op as OperationOutput>::Output,
        ) -> Self::Output
        where
            Self::SelectItems: FromRowData,
        {
            ManyLinkOutput {
                many_output: op.remove(&from_id).unwrap_or_default(),
            }
        }

        type OpInput = Vec<<From::Id as CollectionId>::IdData>;

        fn operation_initialize_input(&self) -> Self::OpInput {
            Vec::new()
        }

        fn operation_fix_on_many(
            &self,
            from_id: &<Self::SelectItems as FromRowData>::RData,
            input: &mut Self::OpInput,
        ) where
            Self::SelectItems: FromRowData,
        {
            input.push(*from_id);
        }

        fn operation_construct(&self, input: Self::OpInput) -> Self::Op
        where
            Self::SelectItems: FromRowData,
        {
            let mut seen = HashSet::new();
            let from_ids = input
                .into_iter()
                .filter(|id| seen.insert(*id))
                .collect();
            FetchOptionalToManyInverseLinked::new(self.clone(), from_ids)
        }
    }
}

pub use post_op::FetchOptionalToManyInverseLinked;

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;

    use crate::{
        collections::Collection,
        connect_in_memory::ConnectInMemory,
        expressions::ColumnEqual,
        extentions::common_expressions::Scoped,
        links::{DefaultRelationKey, relation_optional_to_many_inverse::OptionalToManyInverse},
        operations::{CollectionOutput, LinkedOutput, Operation, fetch_one::FetchOne},
        test_module::{Category, Todo, category, todo},
    };

    #[tokio::test]
    async fn fetch_one_from_category_returns_many_todos() {
        let mut conn = Sqlite::in_memory_connection().await;

        sqlx::query(
            "
            CREATE TABLE Category (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL
            );
            CREATE TABLE Todo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL,
                description TEXT,
                fk_category_def INTEGER,
                FOREIGN KEY (fk_category_def) REFERENCES Category(id)
            );
            INSERT INTO Category (title) VALUES ('work');
            INSERT INTO Todo (title, done, description, fk_category_def) VALUES ('todo_1', 1, NULL, 1);
            INSERT INTO Todo (title, done, description, fk_category_def) VALUES ('todo_2', 0, 'desc', 1);
            ",
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let output = Operation::<Sqlite>::exec_operation(
            FetchOne {
                base: category,
                wheres: ColumnEqual {
                    col: category.id().scoped(),
                    eq: 1,
                },
                links: OptionalToManyInverse {
                    fk_unique_id: DefaultRelationKey,
                    from: category,
                    to: todo,
                },
            },
            &mut conn,
        )
        .await;

        pretty_assertions::assert_eq!(
            output,
            Some(LinkedOutput {
                id: 1,
                attributes: Category {
                    title: "work".to_string(),
                },
                links: crate::operations::ManyLinkOutput {
                    many_output: vec![
                        CollectionOutput {
                            id: 1,
                            attributes: Todo {
                                title: "todo_1".to_string(),
                                done: true,
                                description: None,
                            },
                        },
                        CollectionOutput {
                            id: 2,
                            attributes: Todo {
                                title: "todo_2".to_string(),
                                done: false,
                                description: Some("desc".to_string()),
                            },
                        },
                    ],
                },
            })
        );
    }
}
