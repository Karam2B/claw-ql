#![allow(unexpected_cfgs)]

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct optional_to_many<Id, F, T> {
    pub foriegn_key: Id,
    pub from: F,
    pub to: T,
}

mod impl_on_migrate {
    use crate::{
        collections::{Collection, Id},
        expressions::{
            col_def, foriegn_key, id_constraint, on_delete_set_null, sqlx_type, table_as_expression,
        },
        links::relation_optional_to_many::optional_to_many,
        on_migrate::OnMigrate,
        statements::AddColumn,
    };

    impl<F: Collection + Clone, T: Collection<Id: Id<SqlIdent: ToString>>> OnMigrate
        for optional_to_many<String, F, T>
    {
        type Statements = AddColumn<
            table_as_expression<F>,
            col_def<
                String,
                sqlx_type<Option<i64>>,
                (id_constraint<String, foriegn_key<(on_delete_set_null,)>>,),
            >,
        >;
        fn statments(&self) -> Self::Statements {
            AddColumn {
                table: table_as_expression(self.from.clone()),
                col_def: col_def {
                    name: self.foriegn_key.to_string(),
                    ty: sqlx_type::default(),
                    constraints: (id_constraint(
                        format!("fk_{}", self.foriegn_key),
                        foriegn_key {
                            references_table: self.to.table_name().to_string(),
                            references_col: self.to.id().ident().to_string(),
                            ons: (on_delete_set_null,),
                        },
                    ),),
                },
            }
        }
    }
}

mod impl_fetch_one {
    use sqlx::{ColumnIndex, Database, Row, Type};

    use crate::{
        collections::{Collection, SingleIncremintalInt},
        expressions::{left_join, scoped_column, table},
        extentions::Members,
        from_row::{FromRowAlias, pre_alias},
        links::relation_optional_to_many::optional_to_many,
        operations::{
            CollectionOutput, Operation,
            fetch_one::{LinkFetchOne, SelectStatementExtendableParts},
        },
    };

    impl<S, F, T> LinkFetchOne<S> for optional_to_many<String, F, T>
    where
        T: Collection<Id = SingleIncremintalInt> + Members<S>,
        T: for<'r> FromRowAlias<'r, <S as Database>::Row>,
        F: Collection,
        S: Database,
        i64: for<'q> sqlx::Decode<'q, S> + Type<S>,
        for<'q> &'q str: ColumnIndex<S::Row>,
    {
        type Joins = (left_join,);

        type Wheres = ();

        fn extend_select(
            &self,
        ) -> SelectStatementExtendableParts<
            //
            Vec<scoped_column<String, String>>,
            Self::Joins,
            Self::Wheres,
        > {
            let mut to_members =
                vec![table(self.to.table_name().to_string()).col("id".to_string())];

            to_members.extend(
                Members::members_names(&self.to)
                    .into_iter()
                    .map(|e| return table(self.to.table_name().to_string()).col(e.to_string())),
            );

            SelectStatementExtendableParts {
                non_aggregating_select_items: to_members,
                non_duplicating_joins: (left_join {
                    ft: self.to.table_name().to_string(),
                    fc: "id".to_string(),
                    lt: self.from.table_name().to_string(),
                    lc: self.foriegn_key.clone(),
                },),
                wheres: (),
            }
        }

        type Inner = CollectionOutput<i64, T::Data>;

        type SubOp = ();

        fn sub_op(&self, row: pre_alias<<S as sqlx::Database>::Row>) -> (Self::SubOp, Self::Inner)
        where
            S: sqlx::Database,
        {
            (
                (),
                CollectionOutput {
                    id: row.0.get(format!("{}id", row.1).as_str()),
                    attributes: self.to.pre_alias(row).unwrap(),
                },
            )
        }

        type Output = CollectionOutput<i64, T::Data>;

        fn take(
            self,
            _: <Self::SubOp as Operation<S>>::Output,
            inner: Self::Inner,
        ) -> Self::Output {
            inner
        }
    }
}

// #[cfg(test)]
// mod test {
//     use crate as claw_ql;
//     use claw_ql_macros::{Collection, OnMigrate};

//     use crate::on_migrate::OnMigrate;
//     use crate::query_builder::QueryBuilder;

//     #[derive(OnMigrate, Collection)]
//     pub struct Todo {
//         pub title: String,
//     }

//     fn _assert_stmt_impl() {
//         let mut b = QueryBuilder::<'_, sqlx::Sqlite>::default;

//         OnMigrate::statments(&optional_to_many {
//             foriegn_key: (),
//             from: (),
//             to: (),
//         })
//     }
// }

#[allow(unused)]
#[warn(unused_must_use)]
pub mod impl_dynamic_link {
    use std::{collections::HashMap, sync::Arc};

    use serde::{Deserialize, Serialize};

    use crate::{
        collections::CollectionBasic,
        json_client::json_collection::JsonCollection,
        links::{
            dynamic_link::{CollectionsStore, DynamicLink},
            relation_optional_to_many::optional_to_many,
        },
    };

    pub struct OptionalToManyLinks<DynamicBase> {
        pub all: Vec<optional_to_many<String, DynamicBase, DynamicBase>>,
    }

    #[derive(Deserialize)]
    pub struct RelationOnRequest {
        to: String,
        #[serde(default = "set_def")]
        id: String,
    }

    #[derive(Serialize)]
    pub struct RelationNotFound {
        pub expected_id: String,
        pub expected_from: String,
        pub expected_to: String,
    }

    fn set_def() -> String {
        "default".to_string()
    }

    impl<
        DynamicBase: CollectionBasic + Clone + CollectionsStore<Store = HashMap<String, DynamicBase>>,
        S: 'static,
    > DynamicLink<DynamicBase, S> for OptionalToManyLinks<DynamicBase>
    {
        type OnRequest = optional_to_many<String, DynamicBase, DynamicBase>;

        type OnRequestInput = RelationOnRequest;

        type OnRequestError = RelationNotFound;

        type CreateLinkOk = ();

        type CreateLinkInput = ();

        type CreateLinkError = ();

        type ModifyLinkOk = ();

        type ModifyLinkInput = ();

        type ModifyLinkError = ();

        fn on_request(
            &self,
            base: DynamicBase,
            input: Self::OnRequestInput,
        ) -> Result<Self::OnRequest, Self::OnRequestError> {
            self.all
                .iter()
                .find(|e| {
                    e.foriegn_key == input.id
                        && e.from.table_name_lower_case() == base.table_name_lower_case()
                        && e.to.table_name_lower_case() == input.to
                })
                .cloned()
                .ok_or_else(|| RelationNotFound {
                    expected_id: input.id,
                    expected_from: base.table_name_lower_case().to_string(),
                    expected_to: input.to,
                })
        }

        fn create_link(
            &self,
            base: &HashMap<String, DynamicBase>,
            input: Self::CreateLinkInput,
        ) -> Result<Self::CreateLinkOk, Self::CreateLinkError> {
            todo!()
        }

        fn modify_link(
            &self,
            base: &HashMap<String, DynamicBase>,
            input: Self::ModifyLinkInput,
        ) -> Result<Self::ModifyLinkOk, Self::ModifyLinkError> {
            todo!()
        }
    }
}

#[cfg(feature = "skip_without_comment")]
mod impls {
    // use crate::Accept;
    // use crate::execute::Execute;
    // use crate::json_client::axum_router_mod::HttpError;
    // use crate::json_client::{JsonClient, JsonCollection, JsonError};
    // use crate::links::{Change, LinkedViaId, LinkedViaIds, LiqLink};
    // use crate::operations::CollectionOutput;
    // use crate::operations::delete_one_op::DeleteOneFragment;
    // use crate::prelude::join::left_join;
    // use crate::{
    //     QueryBuilder,
    //     migration::OnMigrate,
    //     operations::{collections::Collection, select_one_op::SelectOneFragment},
    //     prelude::{col, join, stmt::SelectSt},
    // };
    use convert_case::{Case, Casing};
    use serde_json::from_value;
    use sqlx::Pool;
    use sqlx::{ColumnIndex, Decode, Executor, Row, Sqlite, prelude::Type};
    impl<S> optional_to_many<Box<dyn JsonCollection<S>>, Box<dyn JsonCollection<S>>> {
        pub fn liquid() -> Box<dyn LiqLinkExt<S>> {
            Box::new(optional_to_many_liq {
                existing_links: Default::default(),
            })
        }
    }

    impl<F, T> LinkedViaId for optional_to_many<F, T> {}

    #[derive(Clone)]
    #[allow(non_camel_case_types)]
    pub struct optional_to_many_inverse<F, T> {
        pub foriegn_key: String,
        pub from: F,
        pub to: T,
    }

    impl<F, T> LinkedViaIds for optional_to_many_inverse<F, T> {}

    impl<From, To> OnMigrate<Sqlite> for optional_to_many<From, To>
    where
        From: Collection<Sqlite>,
        To: Collection<Sqlite>,
    {
        fn custom_migrate_statements(&self) -> Vec<String> {
            vec![format!(
                "
ALTER TABLE {from_table_name} 
ADD COLUMN {col_name} INT 
REFERENCES {to_table_name} 
(id) 
ON DELETE SET NULL;",
                from_table_name = self.from.table_name(),
                to_table_name = self.to.table_name(),
                col_name = self.foriegn_key,
            )]
        }
    }

    impl<S, From, To> SelectOneFragment<S> for optional_to_many<From, To>
    where
        S: QueryBuilder,
        To: Collection<S, Data: Send + Sync>,
        From: Collection<S, Data: Send + Sync>,
        for<'c> &'c str: ColumnIndex<S::Row>,
        for<'q> i64: Decode<'q, S>,
        i64: Type<S>,
    {
        type Output = Option<CollectionOutput<To::Data>>;
        type Inner = Option<(i64, To::Data)>;

        fn on_select(&mut self, _: &mut Self::Inner, st: &mut SelectSt<S>) {
            st.join(join::left_join {
                foriegn_table: self.to.table_name().to_string(),
                foriegn_column: "id".to_string(),
                local_column: self.foriegn_key.to_string(),
            });
            st.select(col(&self.foriegn_key).table(self.from.table_name()));
            self.to.on_select(st);
        }

        fn from_row(&mut self, data: &mut Self::Inner, row: &S::Row) {
            let id: Option<i64> = row.get(self.foriegn_key.as_str());
            if let Some(id) = id {
                let value = self.to.from_row_scoped(row);
                *data = Some((id, value));
            }
        }

        async fn sub_op<'this>(&'this mut self, _: &'this mut Self::Inner, _: Pool<S>) {
            // no sub_op for optional_to_many
        }

        fn take(self, data: Self::Inner) -> Self::Output {
            data.map(|(id, attr)| CollectionOutput { id, attr })
        }
    }

    impl<S, From, To> DeleteOneFragment<S> for optional_to_many<From, To>
    where
        From: Collection<S, Data: Sync + Send>,
        To: Collection<S, Data: Sync + Send>,
        S: QueryBuilder + Accept<i64>,
        for<'s> &'s str: ColumnIndex<S::Row>,
        SelectSt<S>: Execute<S>,
        S::Fragment: Send,
        S::Context1: Send,
        i64: for<'d> sqlx::Decode<'d, S> + Type<S>,
    {
        type Output = Option<CollectionOutput<To::Data>>;

        type Inner = Option<CollectionOutput<To::Data>>;

        async fn first_sup_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
            &'this mut self,
            data: &'this mut Self::Inner,
            exec: E,
            id: i64,
        ) {
            use sqlx::Row;
            let mut st = SelectSt::init(self.from.table_name());
            self.to.on_select(&mut st);
            st.join(left_join {
                local_column: self.foriegn_key.clone(),
                foriegn_column: "id".to_string(),
                foriegn_table: self.to.table_name().to_string(),
            });
            let alias = format!("{}_id", self.to.table_name());
            st.select(col("id").table(self.to.table_name()).alias(&alias));
            st.where_(col("id").table(self.from.table_name()).eq(id));

            *data = st
                .fetch_optional(exec, |r| {
                    Ok(CollectionOutput {
                        attr: self.to.from_row_scoped(&r),
                        id: r.get(&*alias),
                    })
                })
                .await
                .unwrap();
        }

        fn returning(&self) -> Vec<String> {
            vec![]
        }

        fn from_row(&mut self, _data: &mut Self::Inner, _row: &S::Row) {
            /* no-op */
        }

        fn take(self, data: Self::Inner) -> Self::Output {
            data
        }
    }

    #[derive(Default)]
    pub struct optional_to_many_liq {
        pub existing_links: HashMap<
            String,
            optional_to_many<Box<dyn JsonCollection<Sqlite>>, Box<dyn JsonCollection<Sqlite>>>,
        >,
    }

    #[derive(Deserialize)]
    pub struct CreateLinkRelationInput {
        to: String,
        #[serde(default)]
        id: Option<String>,
    }

    #[derive(Serialize)]
    pub enum CreateLinkRelationErr {
        LinkWithIdExist { id: String },
        CollectionDoesntExist { name: String },
    }
    impl HttpError for CreateLinkRelationErr {
        fn status_code(&self) -> hyper::StatusCode {
            StatusCode::BAD_REQUEST
        }
    }

    #[derive(Serialize)]
    pub enum OnRequestErr {
        LinkWithIdDoesntExist {
            id: String,
        },
        CollectionsAreNotRelated {
            from: String,
            to: String,
            with_id: String,
            type_: String,
        },
    }

    impl HttpError for OnRequestErr {
        fn status_code(&self) -> hyper::StatusCode {
            StatusCode::BAD_REQUEST
        }
    }

    impl LiqLink<Sqlite> for optional_to_many_liq {
        type This = optional_to_many<
            //from, to
            Box<dyn JsonCollection<Sqlite>>,
            Box<dyn JsonCollection<Sqlite>>,
        >;

        type CreateLinkInput = CreateLinkRelationInput;

        type CreateLinkError = CreateLinkRelationErr;

        type CreateLinkOk = ();

        type OnRequestInput = CreateLinkRelationInput;

        type OnRequestError = OnRequestErr;

        fn on_request(
            &self,
            base: &dyn JsonCollection<Sqlite>,
            input: Self::OnRequestInput,
        ) -> Result<Self::This, Self::OnRequestError> {
            let id = if let Some(id) = input.id {
                id
            } else {
                "default".to_string()
            };

            let found = self
                .existing_links
                .get(&id)
                .ok_or(OnRequestErr::LinkWithIdDoesntExist { id: id.clone() })?;

            if found.to.table_name_js() != input.to
                || found.from.table_name_js() != base.table_name_js()
            {
                return Err(OnRequestErr::CollectionsAreNotRelated {
                    from: base.table_name_js().to_string(),
                    to: input.to.to_string(),
                    with_id: id.to_string(),
                    type_: "optional_to_many".to_string(),
                });
            }

            Ok(found.clone())
        }

        fn create_link(
            &mut self,
            collections: &HashMap<String, Box<dyn JsonCollection<Sqlite>>>,
            base: &dyn JsonCollection<Sqlite>,
            input: Self::CreateLinkInput,
        ) -> Result<(Self::CreateLinkOk, Self::This), Self::CreateLinkError> {
            #[rustfmt::skip]
            let id = if let Some(f) = input.id { f } else { "default".to_string() };

            if self.existing_links.contains_key(&id) {
                return Err(CreateLinkRelationErr::LinkWithIdExist { id: id });
            }

            let from = if let Some(found) = collections.get(base.table_name_js()) {
                found.clone_self()
            } else {
                return Err(CreateLinkRelationErr::CollectionDoesntExist {
                    name: base.table_name_js().to_string(),
                });
            };

            let to = if let Some(found) = collections.get(&input.to) {
                found.clone_self()
            } else {
                return Err(CreateLinkRelationErr::CollectionDoesntExist {
                    name: base.table_name_js().to_string(),
                });
            };

            let spec = optional_to_many {
                foriegn_key: format!(
                    "{from}_{to}_{id}",
                    from = from.table_name_js(),
                    to = to.table_name_js()
                ),
                from,
                to,
            };
            self.existing_links.insert(id, spec.clone());

            Ok(((), spec))
        }
    }

    #[derive(Deserialize)]
    pub struct OnRequestRelationInput {
        id: Option<String>,
    }

    #[derive(Serialize)]
    pub enum OnRequestRelationErr {
        RelationWithIdDoesntExist { id: String },
    }
}
