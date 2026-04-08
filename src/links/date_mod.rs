use std::{collections::HashMap, mem, ops::Not, sync::Arc};

use convert_case::Casing;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::from_value;
use sqlx::Sqlite;

use crate::{
    collections::CollectionHandler,
    json_client::{JsonCollection, JsonError, axum_router_mod::HttpError},
    links::{Change, Link, LiqLink},
    migration::OnMigrate,
    operations::select_one_op::{SelectOne, SelectOneFragment},
    prelude::col,
};

pub struct date;

pub struct date_spec<F>(F);
pub trait CollectionIsDated: Clone {}
impl<C: CollectionIsDated> Link<C> for date {
    type Spec = date_spec<C>;
    fn spec(self, b: &C) -> Self::Spec {
        date_spec(b.clone())
    }
}

#[derive(Serialize)]
pub struct DateOutput {
    created_at: String,
    updated_at: String,
}

impl<F: CollectionHandler> SelectOneFragment<Sqlite> for date_spec<F> {
    type Inner = Option<DateOutput>;

    type Output = DateOutput;

    fn on_select(
        &mut self,
        data: &mut Self::Inner,
        st: &mut crate::prelude::stmt::SelectSt<Sqlite>,
    ) {
        st.select(col("created_at"));
        st.select(col("updated_at"));
    }

    fn from_row(&mut self, data: &mut Self::Inner, row: &sqlx::sqlite::SqliteRow) {
        use sqlx::Row;
        *data = Some(DateOutput {
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    fn sub_op<'this>(
        &'this mut self,
        data: &'this mut Self::Inner,
        pool: sqlx::Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + use<'this, F> {
        async {
            // noop
        }
    }

    fn take(self, mut data: Self::Inner) -> Self::Output {
        mem::take(&mut data).unwrap()
    }
}

impl<F: CollectionHandler> OnMigrate<Sqlite> for date_spec<F> {
    fn custom_migrate_statements(&self) -> Vec<String> {
        let table = self.0.table_name().to_case(convert_case::Case::Snake);
        vec![
            format!(
                "ALTER TABLE {table} ADD COLUMN created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP",
            ),
            format!(
                "ALTER TABLE {table} ADD COLUMN updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP",
            ),
            format!(
                "
                CREATE TRIGGER update_{table}_updated_at
                AFTER UPDATE ON {table}
                FOR EACH ROW
                BEGIN
                    UPDATE {table} SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
                END;
                ",
            ),
        ]
    }
}

#[derive(Default)]
pub struct date_liq<S> {
    pub collections: HashMap<String, Box<dyn JsonCollection<S>>>,
}

#[derive(Serialize)]
pub enum DateCreateLinkError {
    CollectionIsDated { name: String },
    CollectionIsNotRegistered { name: String },
}

impl HttpError for DateCreateLinkError {
    fn status_code(&self) -> hyper::StatusCode {
        hyper::StatusCode::BAD_REQUEST
    }
}

impl LiqLink<sqlx::Sqlite> for date_liq<Sqlite> {
    type This = date_spec<Box<dyn JsonCollection<Sqlite>>>;

    type CreateLinkInput = HashMap<(), ()>;
    type CreateLinkOk = ();
    type CreateLinkError = DateCreateLinkError;

    fn create_link(
        &mut self,
        collections: &std::collections::HashMap<
            String,
            Box<dyn crate::json_client::JsonCollection<sqlx::Sqlite>>,
        >,
        base: &dyn crate::json_client::JsonCollection<sqlx::Sqlite>,
        input: Self::CreateLinkInput,
    ) -> Result<(Self::CreateLinkOk, Self::This), Self::CreateLinkError> {
        let base_name = base.table_name_js();
        if self.collections.contains_key(base_name) {
            return Err(DateCreateLinkError::CollectionIsDated {
                name: base_name.to_string(),
            });
        }
        let r = collections.get(base_name).ok_or_else(|| {
            DateCreateLinkError::CollectionIsNotRegistered {
                name: base_name.to_string(),
            }
        })?;

        self.collections
            .insert(base_name.to_string(), r.clone_self());

        Ok(((), date_spec(r.clone_self())))
    }

    type OnRequestInput = HashMap<(), ()>;
    type OnRequestError = CollectionIsNotDated;
    fn on_request(
        &self,
        base: &dyn crate::json_client::JsonCollection<sqlx::Sqlite>,
        input: Self::OnRequestInput,
    ) -> Result<Self::This, Self::OnRequestError> {
        self.collections
            .get(base.table_name_js())
            .map(|e| date_spec(e.clone_self()))
            .ok_or(CollectionIsNotDated {
                name: base.table_name_js().to_string(),
            })
    }
}
#[derive(Serialize)]
pub struct CollectionIsNotDated {
    name: String,
}

impl HttpError for CollectionIsNotDated {
    fn status_code(&self) -> hyper::StatusCode {
        StatusCode::BAD_REQUEST
    }
}
