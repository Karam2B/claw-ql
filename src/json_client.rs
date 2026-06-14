#![allow(unused)]
//! JsonClient
//!     1. relies on
//!

pub mod client_interface {
    use crate::sub_arc::ArcSubStr;

    //*******************
    //*
    //* SupportedType
    //*
    //*******************
    #[derive(Debug)]
    pub enum SupportedType {
        String,
        Boolean,
    }

    //*******************
    //*
    //* AddCollection
    //*
    //*******************
    #[derive(Debug)]
    pub struct AddCollectionInput {
        pub name: ArcSubStr,
        pub fields: Vec<DynamicFieldInput>,
    }

    #[derive(Debug)]
    pub struct DynamicFieldInput {
        pub name: ArcSubStr,
        pub type_info: SupportedType,
        pub is_optional: bool,
    }

    pub type AddCollectionOutput = ();

    //*******************
    //*
    //* InsertOne
    //*
    //*******************
    pub use crate::gen_serde::json_format_side::PartialDeserialize;

    #[derive(Debug)]
    pub struct InsertOneInput {
        pub base: ArcSubStr,
        pub data: PartialDeserialize,
        pub links: Vec<PartialDeserialize>,
    }

    pub type InsertOneOutput = ();

    //*******************
    //*
    //* ClientError
    //*
    //*******************
    #[derive(Debug, Clone)]
    pub struct ClientError {
        pub message: String,
    }

    impl ClientError {
        pub fn disconnected_channel() -> Self {
            Self {
                message: "disconnected channel".into(),
            }
        }
    }

    impl From<&str> for ClientError {
        fn from(message: &str) -> Self {
            Self {
                message: message.to_string(),
            }
        }
    }

    impl From<String> for ClientError {
        fn from(message: String) -> Self {
            Self { message }
        }
    }

    //*******************
    //*
    //* Client
    //*
    //*******************
    pub struct Client {
        pub(crate) sender: tokio::sync::mpsc::UnboundedSender<(
            OperationInput,
            oneshot::Sender<Result<OperationOutput, ClientError>>,
        )>,
    }

    macro_rules! ops {
        ($([$name:ident, $upper_case:ident]),*) => {
            paste::paste!{
                #[derive(Debug)]
                #[non_exhaustive]
                pub enum OperationOutput {
                    $(
                        $upper_case ([<$upper_case Output>]),
                    )*
                }

                #[derive(Debug)]
                #[non_exhaustive]
                pub enum OperationInput {
                    $(
                        $upper_case ([<$upper_case Input>]),
                    )*
                }

                impl Client {
                    $(
                        pub fn
                            $name(&self, input: [<$upper_case Input>])
                        -> impl Future<Output = Result<[<$upper_case Output>], ClientError>> {
                            let (tx, rx) = oneshot::async_channel::<Result<OperationOutput, ClientError>>();
                            self.sender.send((OperationInput::$upper_case(input), tx)).unwrap();
                            async move {
                                let output = rx.await.map_err(|_| ClientError::disconnected_channel())?;
                                let mapp = match output {
                                    Ok(OperationOutput::$upper_case(e)) => Ok(e),
                                    Ok(_) => panic!("invalid mapping"),
                                    Err(e) => Err(e),
                                };

                                return mapp;
                            }
                        }
                    )*
                }

                crate::default_executor!($([$name, $upper_case]),*);
            }
        };
    }

    ops!([add_collection, AddCollection], [insert_one, InsertOne]);
}

mod gen_serde_impls {
    use crate::gen_serde::{
        Deserialize, DeserializeMap, DeserializeSeq, DeserializeSpec, Deserializer, KnownKey,
        json_format_side::StringArcRef,
    };
    use crate::json_client::client_interface::{
        AddCollectionInput, DynamicFieldInput, SupportedType,
    };
    use crate::sub_arc::{ArcSubStr, SubArc};

    impl DeserializeSpec for ArcSubStr {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for ArcSubStr
    where
        S: Deserializer<'de>,
        StringArcRef: Deserialize<'de, S>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let StringArcRef(arc, range) = StringArcRef::deserialize((), serialized)?;
            Ok(SubArc::new(arc, range))
        }
    }

    impl DeserializeSpec for SupportedType {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for SupportedType
    where
        S: Deserializer<'de>,
        String: Deserialize<'de, S>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            match String::deserialize((), serialized)?.as_str() {
                "String" => Ok(SupportedType::String),
                "Boolean" => Ok(SupportedType::Boolean),
                _other => Err(S::Err::from("unsupported SupportedType")),
            }
        }
    }

    impl DeserializeSpec for DynamicFieldInput {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for DynamicFieldInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        SupportedType: Deserialize<'de, S>,
        bool: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let name =
                DeserializeMap::deserialize_pair_known_key(serialized, &mut map, "name", ())?;
            let type_info =
                DeserializeMap::deserialize_pair_known_key(serialized, &mut map, "type_info", ())?;
            let is_optional = DeserializeMap::deserialize_pair_known_key(
                serialized,
                &mut map,
                "is_optional",
                (),
            )?;
            DeserializeMap::finish(serialized, map)?;
            Ok(DynamicFieldInput {
                name,
                type_info,
                is_optional,
            })
        }
    }

    impl DeserializeSpec for AddCollectionInput {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for AddCollectionInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        ArcSubStr: Deserialize<'de, S>,
        Vec<DynamicFieldInput>: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let name =
                DeserializeMap::deserialize_pair_known_key(serialized, &mut map, "name", ())?;
            let fields =
                DeserializeMap::deserialize_pair_known_key(serialized, &mut map, "fields", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(AddCollectionInput { name, fields })
        }
    }

    use crate::gen_serde::json_format_side::PartialDeserialize;
    use crate::json_client::client_interface::InsertOneInput;

    impl DeserializeSpec for InsertOneInput {
        type Handler = ();
    }

    impl<'de, S> Deserialize<'de, S> for InsertOneInput
    where
        S: Deserializer<'de>,
        S: DeserializeMap<'de>,
        S: DeserializeSeq<'de>,
        ArcSubStr: Deserialize<'de, S>,
        PartialDeserialize: Deserialize<'de, S>,
        S: KnownKey<&'static str>,
        S::Err: From<&'static str>,
    {
        fn deserialize(_handler: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
            let mut map = DeserializeMap::start_map(serialized)?;
            let base =
                DeserializeMap::deserialize_pair_known_key(serialized, &mut map, "base", ())?;
            let data =
                DeserializeMap::deserialize_pair_known_key(serialized, &mut map, "data", ())?;
            let links =
                DeserializeMap::deserialize_pair_known_key(serialized, &mut map, "links", ())?;
            DeserializeMap::finish(serialized, map)?;
            Ok(InsertOneInput { base, data, links })
        }
    }

    #[cfg(test)]
    mod tests {
        use std::{marker::PhantomData, sync::Arc};

        use crate::gen_serde::{FromSerializedExt, json_format_side::JsonFormat};
        use crate::json_client::client_interface::{
            AddCollectionInput, DynamicFieldInput, SupportedType,
        };

        #[test]
        fn deserializes_add_collection_input() {
            let json: Arc<str> = Arc::from(
                r#"{
                    "name": "todo",
                    "fields": [
                        {
                            "name": "title",
                            "type_info": "String",
                            "is_optional": false
                        },
                        {
                            "name": "done",
                            "type_info": "Boolean",
                            "is_optional": false
                        },
                        {
                            "name": "description",
                            "type_info": "String",
                            "is_optional": true
                        }
                    ]
                }"#
                .replace('\n', "")
                .replace(' ', ""),
            );

            let input: AddCollectionInput = json
                .deserialize_and_terminate_with(JsonFormat, PhantomData::<AddCollectionInput>)
                .unwrap();

            assert_eq!(input.name.as_str(), "todo");
            assert_eq!(input.fields.len(), 3);

            assert_eq!(input.fields[0].name.as_str(), "title");
            assert!(matches!(input.fields[0].type_info, SupportedType::String));
            assert!(!input.fields[0].is_optional);

            assert_eq!(input.fields[1].name.as_str(), "done");
            assert!(matches!(input.fields[1].type_info, SupportedType::Boolean));
            assert!(!input.fields[1].is_optional);

            assert_eq!(input.fields[2].name.as_str(), "description");
            assert!(matches!(input.fields[2].type_info, SupportedType::String));
            assert!(input.fields[2].is_optional);
        }

        #[test]
        fn deserializes_dynamic_field_input() {
            let json: Arc<str> =
                Arc::from(r#"{"name":"title","type_info":"String","is_optional":false}"#);
            let field: DynamicFieldInput = json
                .deserialize_and_terminate_with(JsonFormat, PhantomData::<DynamicFieldInput>)
                .unwrap();

            assert_eq!(field.name.as_str(), "title");
            assert!(matches!(field.type_info, SupportedType::String));
            assert!(!field.is_optional);
        }

        #[test]
        fn rejects_unsupported_type_info() {
            let json: Arc<str> =
                Arc::from(r#"{"name":"x","type_info":"Integer","is_optional":false}"#);
            let err = json
                .deserialize_and_terminate_with(JsonFormat, PhantomData::<DynamicFieldInput>)
                .unwrap_err();

            assert_eq!(err, "unsupported SupportedType");
        }

        #[test]
        fn rejects_add_collection_input_as_array() {
            let json: Arc<str> = Arc::from(r#"["todo",[]]"#);
            let err = json
                .deserialize_and_terminate_with(JsonFormat, PhantomData::<AddCollectionInput>)
                .unwrap_err();

            assert!(err.contains("expected '{'"));
        }
    }
}

mod add_collection_mod {
    use std::{future::Future, ops::Not, sync::Arc};

    use convert_case::{Case, Casing};
    use sqlx::{Database, IntoArguments};

    use crate::{
        fix_executor::ExecutorTrait,
        json_client::{
            client_interface::{AddCollectionInput, AddCollectionOutput, ClientError},
            sqlx_executor::{DynamicCollection, SqlxExecutorData},
        },
        json_client_v0::database_for_json_client::DatabaseForJsonClient,
        on_migrate::OnMigrate,
        query_builder::{Expression, StatementBuilder},
    };

    pub fn add_collection<S>(
        this: Arc<SqlxExecutorData<S>>,
        input: AddCollectionInput,
    ) -> impl Future<Output = Result<AddCollectionOutput, ClientError>> + 'static + Send + use<S>
    where
        S: DatabaseForJsonClient + Sync,
        bool: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
        std::string::String:
            sqlx::Type<S> + for<'q> sqlx::Encode<'q, S> + for<'d> sqlx::Decode<'d, S>,
        DynamicCollection<S>: OnMigrate<Statements: Expression<'static, S>>,
        for<'a> S::Arguments<'a>: IntoArguments<'a, S>,
    {
        async move {
            let name = input.name.as_str();

            if name.is_empty() {
                Err("name is empty")?;
            }
            if name.is_case(Case::Snake).not() {
                Err("name is not snake_case")?;
            }
            if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                Err("name must not start with a number")?;
            }
            if name.starts_with("ct_") {
                Err("name must not start with ct_")?;
            }
            if name.starts_with("meta_") {
                Err("name must not start with meta_")?;
            }

            for field in &input.fields {
                let field_name = field.name.as_str();
                if field_name.is_empty() {
                    Err("field name is empty")?;
                }
                if field_name == "id" {
                    Err("field name must not be id")?;
                }
                if field_name.starts_with("fk_") {
                    Err("field name must not start with fk_")?;
                }
            }

            let collection_key = name.to_lowercase();

            {
                let collections = this.collections.read().await;
                if collections.get(&collection_key).is_some() {
                    Err("collection already exists")?;
                }
            }

            let dc = DynamicCollection::from(input);

            let mig = StatementBuilder::<S>::new_no_data(dc.statments())
                .ok_or("migration contains bind parameters")?;

            let mut conn = this
                .pool
                .acquire()
                .await
                .map_err(|e| format!("failed to acquire connection: {e}"))?;

            S::execute(&mut conn, mig.as_str())
                .await
                .map_err(|e| format!("migration failed: {e}"))?;

            let mut collections = this.collections.write().await;
            let mut migration = this.migration.write().await;

            migration.push(mig);
            collections.insert(collection_key, tokio::sync::RwLock::new(Arc::new(dc)));

            Ok(())
        }
    }
}

mod insert_one_mod {
    use std::{future::Future, sync::Arc};

    use serde_json::{Map, Value};
    use sqlx::{Database, Row};

    use crate::{
        database_extention::DatabaseExt,
        fix_executor::ExecutorTrait,
        json_client::{
            client_interface::{ClientError, InsertOneInput, InsertOneOutput},
            sqlx_executor::{DynamicCollection, DynamicField, SqlxExecutorData},
            supported_types_vtable::SupportedTypeVTable,
        },
        json_client_v0::database_for_json_client::DatabaseForJsonClient,
        query_builder::StatementBuilder,
    };

    pub fn insert_one<S>(
        this: Arc<SqlxExecutorData<S>>,
        input: InsertOneInput,
    ) -> impl Future<Output = Result<InsertOneOutput, ClientError>> + 'static + Send + use<S>
    where
        S: DatabaseForJsonClient + Sync,
        bool: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
        std::string::String:
            sqlx::Type<S> + for<'q> sqlx::Encode<'q, S> + for<'d> sqlx::Decode<'d, S>,
        for<'a> &'a str: sqlx::ColumnIndex<<S as Database>::Row>,
        i64: for<'r> sqlx::Decode<'r, S> + sqlx::Type<S>,
        for<'a> S::Arguments<'a>: sqlx::IntoArguments<'a, S>,
    {
        async move {
            if !input.links.is_empty() {
                Err("links are not supported yet")?;
            }

            let collection_key = input.base.as_str().to_lowercase();

            let collections = this.collections.read().await;
            let collection_lock = collections
                .get(&collection_key)
                .ok_or_else(|| format!("no collection named {:?}", input.base.as_str()))?
                .read()
                .await;
            let base = collection_lock.clone();
            let locks = vec![collection_lock];

            let _base = base;

            drop(locks);
            todo!()
        }
    }

    fn validate_and_prepare_data<S>(
        collection: &DynamicCollection<S>,
        data: &Map<String, Value>,
    ) -> Result<(), ClientError>
    where
        S: Database,
    {
        for key in data.keys() {
            if collection
                .fields
                .iter()
                .all(|field| field.name.as_ref() != key.as_str())
            {
                Err(format!("unknown field {:?}", key))?;
            }
        }

        for field in &collection.fields {
            let name = field.name.as_ref();
            let value = match data.get(name) {
                Some(value) => value,
                None if field.is_optional => continue,
                None => Err(format!("missing required field {:?}", name))?,
            };
            if value.is_null() && !field.is_optional {
                Err(format!("field {:?} cannot be null", name))?;
            }
            field.type_info.validate_json_value(value)?;
        }
        Ok(())
    }

    fn build_insert_sql<S>(
        collection: &DynamicCollection<S>,
        data: &Map<String, Value>,
    ) -> Result<(String, S::Arguments<'static>), ClientError>
    where
        S: DatabaseExt,
        bool: sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
        String: sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
    {
        let mut sb = StatementBuilder::<'static, S>::default();
        sb.syntax("INSERT INTO ");
        sb.sanitize(collection.name.as_str());
        sb.syntax(" (");
        for (idx, field) in collection.fields.iter().enumerate() {
            if idx > 0 {
                sb.syntax(", ");
            }
            sb.sanitize(field.name.as_ref());
        }
        sb.syntax(") VALUES (");
        for (idx, field) in collection.fields.iter().enumerate() {
            if idx > 0 {
                sb.syntax(", ");
            }
            let value = match data.get(field.name.as_ref()) {
                Some(value) => value,
                None => &Value::Null,
            };
            append_json_bind(&mut sb, field, value)?;
        }
        sb.syntax(") RETURNING ");
        sb.sanitize("id");
        for field in &collection.fields {
            sb.syntax(", ");
            sb.sanitize(field.name.as_ref());
        }
        sb.syntax(";");
        Ok(sb.unwrap())
    }

    fn append_json_bind<S>(
        sb: &mut StatementBuilder<'static, S>,
        field: &DynamicField<S>,
        value: &Value,
    ) -> Result<(), ClientError>
    where
        S: DatabaseExt,
        bool: sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
        String: sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
    {
        use std::any::TypeId;
        let type_id = (field.type_info.type_id)();
        if value.is_null() {
            sb.syntax("NULL");
            return Ok(());
        }
        if type_id == TypeId::of::<String>() {
            sb.bind(value.as_str().ok_or("expected string value")?.to_string());
        } else {
            sb.bind(value.as_bool().ok_or("expected boolean value")?);
        }
        Ok(())
    }

    fn read_attributes_from_row<S>(
        collection: &DynamicCollection<S>,
        row: &S::Row,
    ) -> Result<Map<String, Value>, ClientError>
    where
        S: Database,
        bool: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S>,
        String: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S>,
        for<'a> &'a str: sqlx::ColumnIndex<S::Row>,
    {
        let mut map = Map::new();
        for field in &collection.fields {
            let value = field
                .type_info
                .decode_json_from_row(field.name.as_ref(), row)?;
            map.insert(field.name.to_string(), value);
        }
        Ok(map)
    }
}

mod sqlx_executor {
    use crate::json_client::{
        client_interface::{
            AddCollectionInput, Client, ClientError, OperationInput, OperationOutput, SupportedType,
        },
        supported_types_vtable::SupportedTypeVTable,
    };
    use crate::{
        database_extention::DatabaseExt,
        json_client_v0::database_for_json_client::DatabaseForJsonClient, on_migrate::OnMigrate,
        query_builder::Expression,
    };
    use sqlx::{IntoArguments, Pool};
    use std::{collections::HashMap, marker::PhantomData, sync::Arc};
    use tokio::sync::{RwLock as Trw, mpsc as tokio_mpsc};

    pub struct SqlxExecutor<S: sqlx::Database> {
        pub(crate) reciever: tokio::sync::mpsc::UnboundedReceiver<(
            OperationInput,
            oneshot::Sender<Result<OperationOutput, ClientError>>,
        )>,
        pub(crate) data: Arc<SqlxExecutorData<S>>,
    }

    pub(crate) struct SqlxExecutorData<S: sqlx::Database> {
        pub(crate) collections: Trw<HashMap<String, Trw<Arc<DynamicCollection<S>>>>>,
        pub(crate) migration: Trw<Vec<String>>,
        pub(crate) pool: Pool<S>,
        _s: PhantomData<S>,
    }

    impl Client {
        pub fn new_sqlx_db<S>(pool: Pool<S>) -> (Self, SqlxExecutor<S>)
        where
            S: DatabaseForJsonClient,
            bool: for<'d> sqlx::Decode<'d, S> + sqlx::Type<S> + for<'q> sqlx::Encode<'q, S>,
            std::string::String:
                sqlx::Type<S> + for<'q> sqlx::Encode<'q, S> + for<'d> sqlx::Decode<'d, S>,
            DynamicCollection<S>: OnMigrate<Statements: Expression<'static, S>>,
            for<'a> S::Arguments<'a>: IntoArguments<'a, S>,
        {
            let (sender, reciever) = tokio_mpsc::unbounded_channel::<(
                OperationInput,
                oneshot::Sender<Result<OperationOutput, ClientError>>,
            )>();

            let data = Arc::new(SqlxExecutorData {
                collections: Trw::new(Default::default()),
                migration: Trw::new(Default::default()),
                pool,
                _s: PhantomData,
            });

            (Client { sender }, SqlxExecutor { reciever, data })
        }
    }

    #[derive(Debug)]
    pub(crate) struct DynamicCollection<S> {
        pub(crate) name: String,
        pub(crate) name_lower_case: String,
        pub(crate) fields: Vec<DynamicField<S>>,
    }

    impl<S> From<AddCollectionInput> for DynamicCollection<S> {
        fn from(input: AddCollectionInput) -> Self {
            Self {
                name: input.name.to_string(),
                name_lower_case: input.name.to_lowercase(),
                fields: input
                    .fields
                    .into_iter()
                    .map(|f| DynamicField {
                        name: f.name.to_string().into(),
                        type_info: match f.type_info {
                            SupportedType::String => SupportedTypeVTable::new_as::<String>(),
                            SupportedType::Boolean => SupportedTypeVTable::new_as::<bool>(),
                        },
                        is_optional: f.is_optional,
                    })
                    .collect(),
            }
        }
    }

    #[derive(Debug)]
    pub(crate) struct DynamicField<S> {
        pub(crate) name: Arc<str>,
        pub(crate) type_info: SupportedTypeVTable<S>,
        pub(crate) is_optional: bool,
    }

    mod collection_impls {
        use crate::collections::{Collection, SingleIncremintalInt};

        use super::DynamicCollection;

        impl<S> Collection for DynamicCollection<S> {
            fn table_name(&self) -> &str {
                &self.name
            }

            fn table_name_lower_case(&self) -> &str {
                &self.name_lower_case
            }

            type InputData = ();
            type UpdateData = ();
            type OutputData = ();

            type Id = SingleIncremintalInt<String>;

            fn id(&self) -> Self::Id {
                SingleIncremintalInt(self.name.clone())
            }
        }
    }

    mod std_impls {
        use super::{DynamicCollection, DynamicField};

        impl<S> PartialEq for DynamicCollection<S> {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name && self.fields == other.fields
            }
        }

        impl<S> Eq for DynamicCollection<S> {}

        impl<S> PartialEq for DynamicField<S> {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name
                    && self.type_info == other.type_info
                    && self.is_optional == other.is_optional
            }
        }

        impl<S> Eq for DynamicField<S> {}
    }

    mod impl_on_migrate {
        use super::DynamicCollection;
        use crate::{
            json_client_v0::{
                database_for_json_client::DatabaseForJsonClient,
                dynamic_collection::{
                    DynamicField as OgDynamicField, impl_on_migrate::MigrateDynamicCollection,
                },
            },
            on_migrate::OnMigrate,
        };

        impl<S> OnMigrate for DynamicCollection<S>
        where
            S: DatabaseForJsonClient,
            bool: sqlx::Type<S>,
            std::string::String: sqlx::Type<S>,
        {
            type Statements = MigrateDynamicCollection<S>;

            fn statments(&self) -> Self::Statements {
                MigrateDynamicCollection {
                    name: self.name.clone(),
                    fields: self
                        .fields
                        .iter()
                        .map(|field| OgDynamicField {
                            name: field.name.to_string(),
                            is_optional: field.is_optional,
                            type_info: field.type_info.type_expression(),
                        })
                        .collect(),
                }
            }
        }
    }

    #[macro_export]
    macro_rules! default_executor {
        ($([$name:ident, $upper_case:ident]),*) => {
            impl<S> $crate::json_client::sqlx_executor::SqlxExecutor<S>
            where
                S: $crate::json_client_v0::database_for_json_client::DatabaseForJsonClient + ::std::marker::Sync,
                bool: for<'d> ::sqlx::Decode<'d, S> + ::sqlx::Type<S> + for<'q> ::sqlx::Encode<'q, S>,
                std::string::String:
                    ::sqlx::Type<S> + for<'q> ::sqlx::Encode<'q, S> + for<'d> ::sqlx::Decode<'d, S>,
                $crate::json_client::sqlx_executor::DynamicCollection<S>:
                    $crate::on_migrate::OnMigrate<Statements: $crate::query_builder::Expression<'static, S>>,
                for<'a> S::Arguments<'a>: ::sqlx::IntoArguments<'a, S>,
                for<'a> &'a str: ::sqlx::ColumnIndex<<S as ::sqlx::Database>::Row>,
                i64: for<'r> ::sqlx::Decode<'r, S> + ::sqlx::Type<S>,
            {
                pub fn run(mut self) -> impl Future<Output = ::std::convert::Infallible> {
                    async move {
                        loop {
                            let operation = self.reciever.recv().await.unwrap();

                            paste::paste!{
                            match operation.0 {
                                $(OperationInput::$upper_case(input) => {
                                    let future = $crate::json_client::[<$name _mod>]::[<$name>](self.data.clone(), input);
                                    tokio::spawn(async move {
                                        let resolve_future = future.await;
                                        operation.1.send(resolve_future.map(|e| OperationOutput::$upper_case(e))).unwrap();
                                    });
                                })*
                            }}
                        }
                    }
                }
            }
        };
    }

    #[cfg(test)]
    mod test {
        use std::{marker::PhantomData, sync::Arc};

        use sqlx::Sqlite;

        use crate::{
            connect_in_memory::ConnectInMemory,
            gen_serde::{FromSerializedExt, json_format_side::JsonFormat},
            json_client::{
                client_interface::{AddCollectionInput, Client},
                sqlx_executor::{DynamicCollection, DynamicField, SqlxExecutor},
                supported_types_vtable::SupportedTypeVTable,
            },
        };

        #[test]
        fn test_new_json() {
            let j: Arc<str> = Arc::from(
                "{
        'name': 'todo',
        'fields': [
            {
                'name': 'title',
                'type_info': 'String',
                'is_optional': false,
            },
        ],
    }"
                .replace("\n", "")
                .replace(" ", "")
                .replace("'", "\""),
            );

            let input: AddCollectionInput = j
                .clone()
                .deserialize_and_terminate_with(JsonFormat, PhantomData::<AddCollectionInput>)
                .unwrap();

            assert_eq!(input.name.as_str(), "todo");
            assert_eq!(input.fields[0].name.as_str(), "title");

            let dc: DynamicCollection<Sqlite> = input.into();

            pretty_assertions::assert_eq!(
                dc,
                DynamicCollection::<Sqlite> {
                    name: "todo".to_string(),
                    name_lower_case: "todo".to_string(),
                    fields: vec![DynamicField {
                        name: "title".to_string().into(),
                        is_optional: false,
                        type_info: SupportedTypeVTable::new_as::<String>(),
                    },],
                }
            );
        }

        #[tokio::test]
        async fn add_collection_creates_table_and_registers_collection() {
            let pool = Sqlite::connect_in_memory().await;
            let (client, executor) = Client::new_sqlx_db(pool.clone());
            let data = executor.data.clone();
            tokio::spawn(executor.run());

            let json: Arc<str> = Arc::from(
                r#"{"name":"todo","fields":[{"name":"title","type_info":"String","is_optional":false}]}"#,
            );
            let input: AddCollectionInput = json
                .deserialize_and_terminate_with(JsonFormat, PhantomData::<AddCollectionInput>)
                .unwrap();

            client.add_collection(input).await.unwrap();

            let collections = data.collections.read().await;
            assert!(collections.contains_key("todo"));

            let table: (String,) = sqlx::query_as(
                r#"SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'todo'"#,
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(table.0, "todo");
        }

        #[tokio::test]
        async fn insert_one_inserts_row() {
            use std::{marker::PhantomData, sync::Arc};

            use serde_json::json;

            use crate::{
                connect_in_memory::ConnectInMemory,
                gen_serde::{FromSerializedExt, json_format_side::JsonFormat},
                json_client::client_interface::{AddCollectionInput, Client, InsertOneInput},
                json_client::sqlx_executor::SqlxExecutor,
            };

            let pool = Sqlite::connect_in_memory().await;
            let (client, executor) = Client::new_sqlx_db(pool.clone());
            let data = executor.data.clone();
            tokio::spawn(executor.run());

            let add_json: Arc<str> = Arc::from(
                r#"{"name":"todo","fields":[{"name":"title","type_info":"String","is_optional":false},{"name":"done","type_info":"Boolean","is_optional":false},{"name":"description","type_info":"String","is_optional":true}]}"#,
            );
            let add_input: AddCollectionInput = add_json
                .deserialize_and_terminate_with(JsonFormat, PhantomData::<AddCollectionInput>)
                .unwrap();
            client.add_collection(add_input).await.unwrap();

            let insert_json: Arc<str> = Arc::from(
                r#"{"base":"todo","data":{"title":"new_todo","done":false,"description":"description_6"},"links":[]}"#,
            );
            let insert_input: InsertOneInput = insert_json
                .deserialize_and_terminate_with(JsonFormat, PhantomData::<InsertOneInput>)
                .unwrap();

            let out = client.insert_one(insert_input).await.unwrap();

            assert_eq!(out.id, 1);
            assert_eq!(
                out.attributes.get("title").and_then(|v| v.as_str()),
                Some("new_todo")
            );
            assert_eq!(
                out.attributes.get("done").and_then(|v| v.as_bool()),
                Some(false)
            );
            assert_eq!(
                out.attributes.get("description").and_then(|v| v.as_str()),
                Some("description_6")
            );
            assert!(out.links.is_empty());

            let collections = data.collections.read().await;
            assert!(collections.contains_key("todo"));
        }
    }
}

mod supported_types_vtable {
    use core::fmt;
    use std::{any::TypeId, marker::PhantomData};

    use crate::{
        database_extention::DatabaseExt,
        query_builder::{SyntaxAsType, functional_expr::BoxedExpression},
    };

    pub struct SupportedTypeVTable<S> {
        pub type_name: fn() -> &'static str,
        pub type_id: fn() -> TypeId,
        _s: PhantomData<S>,
    }

    impl<S> fmt::Debug for SupportedTypeVTable<S> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "SupportedTypeVTable({})", (self.type_name)())
        }
    }

    impl<S> PartialEq for SupportedTypeVTable<S> {
        fn eq(&self, other: &Self) -> bool {
            (self.type_name)() == (other.type_name)()
        }
    }

    impl<S> Eq for SupportedTypeVTable<S> {}

    impl<S> SupportedTypeVTable<S> {
        pub fn new_as<T>() -> Self
        where
            T: Send + Sync + 'static,
        {
            let type_name = || {
                return std::any::type_name::<T>();
            };

            let type_id = || {
                return TypeId::of::<T>();
            };

            Self {
                type_name,
                type_id,
                _s: PhantomData,
            }
        }

        pub fn type_expression(&self) -> Box<dyn BoxedExpression<S> + Send>
        where
            S: DatabaseExt,
            bool: sqlx::Type<S>,
            std::string::String: sqlx::Type<S>,
        {
            if (self.type_id)() == TypeId::of::<String>() {
                Box::new(SyntaxAsType::<String>(PhantomData))
            } else if (self.type_id)() == TypeId::of::<bool>() {
                Box::new(SyntaxAsType::<bool>(PhantomData))
            } else {
                panic!(
                    "unsupported SupportedTypeVTable type: {}",
                    (self.type_name)()
                );
            }
        }

        pub fn validate_json_value(&self, value: &serde_json::Value) -> Result<(), String> {
            use std::any::TypeId;
            if value.is_null() {
                return Ok(());
            }
            if (self.type_id)() == TypeId::of::<String>() {
                value
                    .as_str()
                    .ok_or_else(|| format!("expected string for {}", (self.type_name)()))?;
            } else if (self.type_id)() == TypeId::of::<bool>() {
                value
                    .as_bool()
                    .ok_or_else(|| format!("expected boolean for {}", (self.type_name)()))?;
            } else {
                return Err(format!("unsupported type {}", (self.type_name)()));
            }
            Ok(())
        }

        pub fn decode_json_from_row(
            &self,
            name: &str,
            row: &S::Row,
        ) -> Result<serde_json::Value, String>
        where
            S: sqlx::Database,
            bool: for<'r> sqlx::Decode<'r, S> + sqlx::Type<S>,
            String: for<'r> sqlx::Decode<'r, S> + sqlx::Type<S>,
            for<'a> &'a str: sqlx::ColumnIndex<S::Row>,
        {
            use sqlx::Row;
            use std::any::TypeId;
            if (self.type_id)() == TypeId::of::<String>() {
                let v: Option<String> = row
                    .try_get(name)
                    .map_err(|e| format!("decode {:?}: {e}", name))?;
                return Ok(match v {
                    None => serde_json::Value::Null,
                    Some(s) => serde_json::Value::String(s),
                });
            }
            if (self.type_id)() == TypeId::of::<bool>() {
                let v: Option<bool> = row
                    .try_get(name)
                    .map_err(|e| format!("decode {:?}: {e}", name))?;
                return Ok(match v {
                    None => serde_json::Value::Null,
                    Some(b) => serde_json::Value::Bool(b),
                });
            }
            Err(format!("unsupported type {}", (self.type_name)()))
        }
    }
}
