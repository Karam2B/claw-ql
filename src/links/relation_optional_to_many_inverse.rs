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

mod impl_link_fetch {
    use std::collections::HashSet;

    use crate::{
        collections::{Collection, CollectionId, SingleColumnId},
        extentions::{Members, common_expressions::{Aliased, TableNameExpression}},
        from_row::FromRowData,
        links::relation_optional_to_many_inverse::OptionalToManyInverse,
        operations::{
            CollectionOutput, ManyLinkOutput, OperationOutput, fetch_many::LinkFetch,
            fetch_linked_records::{
                FetchOptionalToManyInverseLinked, OptionalToManyInverseLinkedMap,
            },
        },
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

pub use crate::operations::fetch_linked_records::FetchOptionalToManyInverseLinked;

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
