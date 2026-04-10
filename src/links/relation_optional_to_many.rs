#![allow(unexpected_cfgs)]
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct optional_to_many<F, T> {
    pub foriegn_key: String,
    pub from: F,
    pub to: T,
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
