
use crate::{
    extentions::common_expressions::Aliased,
    from_row::{
        FromRowAlias, FromRowData, FromRowError, RowPostAliased, RowPreAliased, RowTwoAliased,
    },
    sqlx_query_builder::{basic_expressions::ManyFlat, trait_objects::ManyBoxedExpressions},
};
use sqlx::Database;
use std::any::Any;

pub trait SelectItemsTraitObject<S, CastFromRowResult>: Send {
    fn str_alias_erase(&self, alias: &'static str) -> Box<dyn ManyBoxedExpressions<S> + Send>;

    fn num_alias_erase(
        &self,
        num: usize,
        alias: &'static str,
    ) -> Box<dyn ManyBoxedExpressions<S> + Send>;

    fn no_alias_2<'r>(&self, row: &'r S::Row) -> Result<Box<dyn Any + Send>, FromRowError>
    where
        S: Database;

    fn pre_alias_2<'r>(
        &self,
        row: RowPreAliased<'r, S::Row>,
    ) -> Result<Box<dyn Any + Send>, FromRowError>
    where
        S: Database,
        S::Row: sqlx::Row;

    fn post_alias_2<'r>(
        &self,
        row: RowPostAliased<'r, S::Row>,
    ) -> Result<Box<dyn Any + Send>, FromRowError>
    where
        S: Database,
        S::Row: sqlx::Row;

    fn two_alias_2<'r>(
        &self,
        row: RowTwoAliased<'r, S::Row>,
    ) -> Result<Box<dyn Any + Send>, FromRowError>
    where
        S: Database,
        S::Row: sqlx::Row;
}

pub struct ToImplSelectItems<Se, CastFromRowResult> {
    pub select_items: Se,
    pub cast_from_row_result: CastFromRowResult,
}

impl<Se, S> SelectItemsTraitObject<S, ()> for ToImplSelectItems<Se, ()>
where
    Se: Send,
    Se: Aliased<Aliased: 'static + Send + ManyBoxedExpressions<S>>,
    Se: Aliased<NumAliased: 'static + Send + ManyBoxedExpressions<S>>,
    Se: for<'r> FromRowAlias<'r, S::Row>,
    Se: FromRowData<RData: Send + 'static>,
    S: Database,
{
    fn str_alias_erase(&self, alias: &'static str) -> Box<dyn ManyBoxedExpressions<S> + Send> {
        Box::new(self.select_items.aliased(alias))
    }
    fn num_alias_erase(
        &self,
        num: usize,
        alias: &'static str,
    ) -> Box<dyn ManyBoxedExpressions<S> + Send> {
        Box::new(self.select_items.num_aliased(num, alias))
    }
    fn no_alias_2<'r>(&self, row: &'r S::Row) -> Result<Box<dyn Any + Send>, FromRowError> {
        Ok(Box::new(self.select_items.no_alias(row)?))
    }
    fn pre_alias_2<'r>(
        &self,
        row: RowPreAliased<'r, S::Row>,
    ) -> Result<Box<dyn Any + Send>, FromRowError> {
        let ret = self.select_items.pre_alias(row)?;
        Ok(Box::new(ret))
    }
    fn post_alias_2<'r>(
        &self,
        row: RowPostAliased<'r, S::Row>,
    ) -> Result<Box<dyn Any + Send>, FromRowError> {
        Ok(Box::new(self.select_items.post_alias(row)?))
    }
    fn two_alias_2<'r>(
        &self,
        row: RowTwoAliased<'r, S::Row>,
    ) -> Result<Box<dyn Any + Send>, FromRowError> {
        Ok(Box::new(self.select_items.two_alias(row)?))
    }
}

impl<'r, S, C> Aliased for Box<dyn SelectItemsTraitObject<S, C> + 'r> {
    type Aliased = Box<dyn ManyBoxedExpressions<S> + Send>;

    fn aliased(&self, alias: &'static str) -> Self::Aliased {
        self.str_alias_erase(alias)
    }

    type NumAliased = Box<dyn ManyBoxedExpressions<S> + Send>;
    fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
        self.num_alias_erase(num, alias)
    }
}

impl<'r, S, C> Aliased for Vec<Box<dyn SelectItemsTraitObject<S, C> + 'r>> {
    type Aliased = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

    fn aliased(&self, alias: &'static str) -> Self::Aliased {
        ManyFlat(
            self.iter()
                .enumerate()
                .map(|(i, each)| each.num_alias_erase(i, alias))
                .collect::<Vec<_>>(),
        )
    }

    type NumAliased = ManyFlat<Box<dyn ManyBoxedExpressions<S> + Send>>;
    fn num_aliased(&self, _: usize, _: &'static str) -> Self::NumAliased {
        panic!("bug: nesting where it was not expected");
    }
}

impl<'r, S> FromRowData for Box<dyn SelectItemsTraitObject<S, ()> + 'r> {
    type RData = Box<dyn Any + Send>;
}

impl<'r, 'b, S: Database> FromRowAlias<'r, S::Row> for Box<dyn SelectItemsTraitObject<S, ()> + 'b> {
    fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
        Ok(self.no_alias_2(row)?)
    }
    fn pre_alias(&self, row: RowPreAliased<'r, S::Row>) -> Result<Self::RData, FromRowError> {
        Ok(self.pre_alias_2(row)?)
    }
    fn post_alias(&self, row: RowPostAliased<'r, S::Row>) -> Result<Self::RData, FromRowError> {
        Ok(self.post_alias_2(row)?)
    }

    fn two_alias(&self, row: RowTwoAliased<'r, S::Row>) -> Result<Self::RData, FromRowError>
    where
        S::Row: sqlx::Row,
    {
        Ok(self.two_alias_2(row)?)
    }
}
impl<'r, S> FromRowData for Vec<Box<dyn SelectItemsTraitObject<S, ()> + 'r>> {
    type RData = Vec<Box<dyn Any + Send>>;
}

impl<'r, 'b, S: Database> FromRowAlias<'r, S::Row>
    for Vec<Box<dyn SelectItemsTraitObject<S, ()> + 'b>>
{
    fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
        let mut v = vec![];
        for (i, each) in self.iter().enumerate() {
            v.push(each.two_alias(RowTwoAliased {
                row: row,
                str_alias: "",
                num_alias: Some(i),
            })?);
        }
        Ok(v)
    }
    fn pre_alias(&self, row: RowPreAliased<'r, S::Row>) -> Result<Self::RData, FromRowError> {
        let mut v = vec![];
        for (i, each) in self.iter().enumerate() {
            v.push(each.two_alias(RowTwoAliased {
                row: row.row,
                str_alias: row.alias,
                num_alias: Some(i),
            })?);
        }
        Ok(v)
    }
    fn post_alias(&self, _: RowPostAliased<'r, S::Row>) -> Result<Self::RData, FromRowError> {
        panic!("in the process of deprecating this method");
    }

    fn two_alias(&self, _: RowTwoAliased<'r, S::Row>) -> Result<Self::RData, FromRowError>
    where
        S::Row: sqlx::Row,
    {
        panic!("nesting where it was not expected");
    }
}

#[cfg(test)]
mod test {
    use crate::{
        connect_in_memory::ConnectInMemory,
        links::{DefaultRelationKey, relation_optional_to_many::OptionalToMany},
        operations::{Operation, fetch_many::FetchMany},
        test_module::{category, todo},
    };
    use serde_json::json;
    use sqlx::Sqlite;

    #[tokio::test]
    async fn test_ref_link() {
        let mut db = Sqlite::in_memory_connection().await;

        sqlx::query(
            "
        CREATE TABLE Category ( 
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT
        );
        CREATE TABLE Todo ( 
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT, 
            done BOOLEAN, 
            description TEXT, 
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            fk_category_def INTEGER, FOREIGN KEY (fk_category_def) REFERENCES Category(id)
        );

        INSERT INTO Category (title) VALUES 
        ('category_1'), ('category_2'), ('category_3');

        INSERT INTO Todo
            (title, done, description, fk_category_def, created_at, updated_at)
        VALUES
            ('first_todo', true, 'description_1', 1, 'test_0', 'test_1'),
            ('second_todo', false, 'description_2', NULL, 'test_2', 'test_3'),
            ('third_todo', true, 'description_3', 2, 'test_4', 'test_5'),
            ('fourth_todo', false, 'description_4', 2, 'test_6', 'test_7');
    ",
        )
        .execute(&mut db)
        .await
        .unwrap();

        let output = Operation::<Sqlite>::exec_operation(
            FetchMany {
                base: todo,
                wheres: (),
                links: OptionalToMany {
                    from: todo,
                    to: category,
                    fk_unique_id: DefaultRelationKey,
                },
                cursor_order_by: (),
                cursor_first_item: None::<(i64, ())>,
                limit: 10,
            },
            &mut db,
        )
        .await;

        pretty_assertions::assert_eq!(
            serde_json::to_value(output).unwrap(),
            json!({
                "items": [
                    {
                        "id": 1,
                        "attributes": {
                            "title": "first_todo",
                            "done": true,
                            "description": "description_1",
                        },
                        "links": {
                            "id": 1,
                            "attributes": {
                                "title": "category_1",
                            }
                        }
                    },
                    {
                        "id": 2,
                        "attributes": {
                            "title": "second_todo",
                            "done": false,
                            "description": "description_2",
                        },
                        "links": null
                    },
                    {
                        "id": 3,
                        "attributes": {
                            "title": "third_todo",
                            "done": true,
                            "description": "description_3",
                        },
                        "links": {
                            "id": 2,
                            "attributes": {
                                "title": "category_2",
                            }
                        }
                    },
                    {
                        "id": 4,
                        "attributes": {
                            "title": "fourth_todo",
                            "done": false,
                            "description": "description_4",
                        },
                        "links": {
                            "id": 2,
                            "attributes": {
                                "title": "category_2",
                            }
                        }
                    }
                ],
                "next_item": null,
            })
        );
    }
}
