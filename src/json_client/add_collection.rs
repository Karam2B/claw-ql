use hyper::StatusCode;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json, to_value};
use sqlx::{ColumnIndex, Database, Decode, TypeInfo};
use sqlx::{Encode, IntoArguments, Pool, Sqlite, Type};
use std::{collections::HashMap, marker::PhantomData};

use crate::collections::CollectionHandler;
use crate::prelude::primary_key;
use crate::prelude::stmt::CreateTableSt;
use crate::statements::create_table_st::header;
use crate::{BindItem, Buildable, ColumPositionConstraint};
use crate::{
    QueryBuilder,
    json_client::{JsonClient, JsonCollection, axum_router_mod::HttpError},
    migration::OnMigrate,
    prelude::stmt::SelectSt,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddCollectionBody {
    pub name: String,
    pub fields: HashMap<String, FieldInJson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldInJson {
    pub typeid: String,
    pub optional: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddCollectionRes {}

#[derive(Debug, PartialEq, Eq)]
pub struct CollectionExist(String);

impl HttpError for CollectionExist {
    fn status_code(&self) -> StatusCode {
        StatusCode::CONFLICT
    }
    fn sub_code(&self) -> Option<&'static str> {
        Some("collection_exist")
    }
    fn sub_message(&self) -> Option<String> {
        Some(format!("collection {} exist", self.0))
    }
}

pub struct DynamicField<S> {
    pub name: String,
    pub is_optional: bool,
    pub type_info: Box<dyn LiqType<S>>,
}

impl<S> Clone for DynamicField<S> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            is_optional: self.is_optional,
            type_info: self.type_info.clone_self(),
        }
    }
}

pub struct DynamicTypeConstraint(String);

impl ColumPositionConstraint for DynamicTypeConstraint {}
impl<S: QueryBuilder> BindItem<S> for DynamicTypeConstraint {
    fn bind_item(
        self,
        ctx: &mut <S as QueryBuilder>::Context1,
    ) -> impl FnOnce(&mut <S as QueryBuilder>::Context2) -> String + 'static + use<S> {
        |s| "".to_string()
    }
}

pub trait LiqType<S>: Send + Sync {
    fn typeid(&self) -> String;
    fn clone_self(&self) -> Box<dyn LiqType<S>>;
    fn on_insert(
        &self,
        val: Value,
        stmt: &mut crate::prelude::stmt::InsertOneSt<S>,
        name: &str,
    ) -> Result<(), String>
    where
        S: Database;

    fn from_row_optional(&self, name: &str, row: &S::Row) -> Value
    where
        S: Database;
    fn typeinfo(&self) -> TypeInfoS;
}

pub trait LiqTypeDeprecate<S> {
    fn typeinfo(&self) -> TypeInfoS;
}
pub struct TypeInfoS {
    is_null: bool,
    is_void: bool,
    name: String,
}
impl<S, T> LiqTypeDeprecate<S> for PhantomData<T>
where
    S: Database,
    T: sqlx::Type<S>,
{
    fn typeinfo(&self) -> TypeInfoS {
        use sqlx::TypeInfo;
        let s = T::type_info();
        let is_null = s.is_null();
        let is_void = s.is_void();
        let name = s.name();
        TypeInfoS {
            is_null,
            is_void,
            name: name.to_string(),
        }
    }
}

impl ColumPositionConstraint for TypeInfoS {}
impl<S: QueryBuilder> BindItem<S> for TypeInfoS {
    fn bind_item(
        self,
        ctx: &mut <S as QueryBuilder>::Context1,
    ) -> impl FnOnce(&mut <S as QueryBuilder>::Context2) -> String + 'static + use<S> {
        move |e| format!("{}", self.name)
    }
}
// impl<S>

pub trait SerializableAny {
    fn typeid(&self) -> String;
}

impl SerializableAny for PhantomData<i32> {
    fn typeid(&self) -> String {
        "core::i32".to_string()
    }
}
impl SerializableAny for PhantomData<bool> {
    fn typeid(&self) -> String {
        "core::boolean".to_string()
    }
}
impl SerializableAny for PhantomData<String> {
    fn typeid(&self) -> String {
        "core::string".to_string()
    }
}

// T here should never be Option<..>
pub struct DynamicTypeWASMMod<S>(S);
impl<S: Send + Sync> LiqType<S> for DynamicTypeWASMMod<S> {
    fn typeid(&self) -> String {
        todo!()
    }

    fn clone_self(&self) -> Box<dyn LiqType<S>> {
        todo!()
    }

    fn on_insert(
        &self,
        val: Value,
        stmt: &mut crate::prelude::stmt::InsertOneSt<S>,
        name: &str,
    ) -> Result<(), String>
    where
        S: Database,
    {
        todo!()
    }

    fn typeinfo(&self) -> TypeInfoS {
        todo!()
    }
    fn from_row_optional(&self, name: &str, row: &<S>::Row) -> Value
    where
        S: Database,
    {
        todo!()
    }
}

impl<S, T> LiqType<S> for PhantomData<T>
where
    for<'a> &'a str: ColumnIndex<S::Row>,
    S: Database,
    T: 'static
        + for<'a> Decode<'a, S>
        + Encode<'static, S>
        + Type<S>
        + Send
        + Sync
        + DeserializeOwned
        + Serialize,
    Self: SerializableAny,
{
    fn typeid(&self) -> String {
        <Self as SerializableAny>::typeid(self)
    }
    fn clone_self(&self) -> Box<dyn LiqType<S>> {
        Box::new(PhantomData::<T>)
    }
    fn on_insert(
        &self,
        val: Value,
        stmt: &mut crate::prelude::stmt::InsertOneSt<S>,
        name: &str,
    ) -> Result<(), String>
    where
        S: Database,
    {
        let t = serde_json::from_value::<T>(val).map_err(|e| e.to_string())?;
        stmt.col(name.to_string(), t);
        Ok(())
    }
    fn typeinfo(&self) -> TypeInfoS {
        use sqlx::TypeInfo;
        let s = T::type_info();
        let is_null = s.is_null();
        let is_void = s.is_void();
        let name = s.name();
        TypeInfoS {
            is_null,
            is_void,
            name: name.to_string(),
        }
    }
    fn from_row_optional(&self, name: &str, row: &S::Row) -> Value
    where
        S: Database,
    {
        use sqlx::Row;

        let ret: Option<T> = row
            .try_get(name)
            .expect("shouldn't typeing error be outruled at init-time");

        let va = serde_json::to_value(ret).expect("shoudn't serializing error be outruled?");

        va
    }
}

pub struct DynamicCollection<S: QueryBuilder> {
    pub name: String,
    pub fields: Vec<DynamicField<S>>,
}

impl<S: QueryBuilder> Clone for DynamicCollection<S> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            fields: self.fields.clone(),
        }
    }
}

impl OnMigrate<Sqlite> for DynamicCollection<Sqlite> {
    fn custom_migrate_statements(&self) -> Vec<String> {
        let mut stmt = CreateTableSt::<Sqlite>::init(header::create, &self.name);
        stmt.column_def("id", primary_key::<Sqlite>());
        for each in self.fields.iter() {
            stmt.column_def(&each.name, each.type_info.typeinfo());
        }
        vec![Buildable::build(stmt).0]
    }
}

// impl<S: QueryBuilder> CollectionBasic for DynamicCollection<S> {
//     fn table_name(&self) -> &'static str {
//         todo!()
//     }

//     fn table_name_lower_case(&self) -> &'static str {
//         todo!()
//     }

//     fn members(&self) -> Vec<String> {
//         todo!()
//     }

//     type LinkedData = DynamicCollection<S>;
// }

impl<S: QueryBuilder + Sync> JsonCollection<S> for DynamicCollection<S>
where
    for<'a> &'a str: ColumnIndex<S::Row>,
{
    fn clone_self(&self) -> Box<dyn JsonCollection<S>> {
        let s = DynamicCollection::<S>::clone(self);
        Box::new(s)
    }
    fn table_name_js(&self) -> &str {
        &self.name
    }

    fn members_js(&self) -> Vec<String> {
        self.fields.iter().map(|e| e.name.to_string()).collect()
    }

    fn on_select(&self, stmt: &mut crate::prelude::stmt::SelectSt<S>)
    where
        S: crate::QueryBuilder,
    {
        for field in self.fields.iter() {
            stmt.select(
                crate::prelude::col(&field.name)
                    .table(&self.name)
                    .alias(&format!("{}_{}", self.table_name_js(), field.name)),
            );
        }
    }

    fn on_insert(
        &self,
        this: serde_json::Value,
        stmt: &mut crate::prelude::stmt::InsertOneSt<S>,
    ) -> Result<(), String>
    where
        S: sqlx::Database,
    {
        let this_obj = this.as_object().ok_or("failed to parse to object")?;
        for field in self.fields.iter() {
            field.type_info.on_insert(
                this_obj
                    .get(&field.name)
                    .cloned()
                    .ok_or("err".to_string())?,
                stmt,
                &field.name,
            )?;
        }
        todo!()
    }

    fn on_update(
        &self,
        this: serde_json::Value,
        stmt: &mut crate::prelude::macro_derive_collection::UpdateSt<S>,
    ) -> Result<(), String>
    where
        S: crate::QueryBuilder,
    {
        todo!()
    }

    fn from_row_noscope(&self, row: &<S>::Row) -> serde_json::Value
    where
        S: Database,
    {
        use sqlx::Row;
        panic!("rows{:?}", row.columns());
        for field in self.fields.iter() {
            let typei = &field.type_info;
            let ret = field.type_info.from_row_optional(&field.name, row);
        }
        todo!()
    }

    #[track_caller]
    fn from_row_scoped(&self, row: &<S>::Row) -> serde_json::Value
    where
        S: Database,
    {
        use sqlx::Row;
        let table_name = &self.name;
        let mut map = serde_json::Map::default();
        for field in self.fields.iter() {
            let name = &field.name;
            let typei = &field.type_info;
            let ret = field
                .type_info
                .from_row_optional(&format!("{}_{name}", table_name), row);
            let inserted = map.insert(field.name.clone(), ret);
            if inserted.is_some() {
                panic!("map should be empty")
            }
        }
        serde_json::to_value(map).unwrap()
    }
}

impl JsonClient<Sqlite> {
    // #[axum::debug_handler]
    pub async fn add_collection(&mut self, body: AddCollectionBody) -> Result<(), CollectionExist> {
        let s = self.collections.get_mut(&body.name);
        if s.is_some() {
            return Err(CollectionExist(body.name));
        }

        // sqlx::query("CREATE TABLE {}")
        let collection = DynamicCollection {
            name: body.name.clone(),
            fields: body
                .fields
                .into_iter()
                .map(|(name, field_i)| DynamicField {
                    name,
                    is_optional: field_i.optional,
                    type_info: self
                        .type_extentions
                        .get(&field_i.typeid)
                        .expect("type must register before use")
                        .clone_self(),
                })
                .collect(),
        };

        let stmt = collection.custom_migrate_statements();

        for each in stmt {
            sqlx::query(&each).execute(&self.db).await.unwrap();
        }

        self.collections.insert(body.name, Box::new(collection));

        Ok(())
    }
}
