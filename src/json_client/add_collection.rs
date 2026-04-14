use crate::collections::SingleIncremintalInt;
use crate::database_extention::DatabaseExt;
use crate::json_client::database_for_json_client::DatabaseForJsonClient;
use crate::json_client::type_t_trait::TypeT;
use crate::query_builder::Expression;
use crate::query_builder::QueryBuilder;
use crate::query_builder::functional_expr::ManyFlat;
use convert_case::Case;
use convert_case::Casing;
use serde::{Deserialize, Serialize};
use sqlx::Executor;
use sqlx::IntoArguments;
use std::ops::Not;
use std::sync::Arc;

impl<S> Clone for DynamicCollection<S> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            name_lower_case: self.name_lower_case.clone(),
            fields: self
                .fields
                .iter()
                .map(|e| DynamicField {
                    name: e.name.clone(),
                    is_optional: e.is_optional,
                    type_info: e.type_info.clone_self(),
                })
                .collect(),
        }
    }
}

mod dynamic_field_impl {
    use std::ops::Not;

    use super::*;
    use crate::query_builder::Expression;
    use crate::query_builder::OpExpression;
    use crate::query_builder::QueryBuilder;
    use sqlx::Sqlite;

    impl OpExpression for DynamicField<TypeSpec> {}
    impl<'q> Expression<'q, Sqlite> for DynamicField<TypeSpec> {
        fn expression(self, arg: &mut QueryBuilder<'q, Sqlite>) {
            arg.sanitize(self.name.as_str());
            arg.syntax(&" ");

            match self.type_info {
                TypeSpec::String => arg.syntax(&"TEXT"),
                TypeSpec::Boolean => arg.syntax(&"BOOLEAN"),
            }
            if self.is_optional.not() {
                arg.syntax(&" NOT NULL");
            }
        }
    }
}

mod sqlx_extention_impls {
    use std::marker::PhantomData;

    use crate::json_client::to_bind_trait::ToBind;

    use super::DatabaseForJsonClient;
    use sqlx::Sqlite;

    impl DatabaseForJsonClient for Sqlite {
        fn string_to_bind(str: String) -> Box<dyn ToBind<Self>> {
            Box::new(str)
        }
        fn type_info_default() -> Self::TypeInfo {
            todo!()
        }
        fn support_string() -> Box<dyn super::TypeT<Self> + Send + Sync> {
            Box::new(PhantomData::<String>)
        }
        fn support_boolean() -> Box<dyn super::TypeT<Self> + Send + Sync> {
            Box::new(PhantomData::<bool>)
        }
    }
}

pub mod dynamic_collection_impl {
    use std::{collections::HashMap, ops::Not};

    use claw_ql_macros::skip;
    use convert_case::Case;
    use serde::de;
    use serde_json::from_value;
    use sqlx::{Database, Encode, Type, encode};

    use crate::{
        collections::{Collection, ValidateCollection},
        expressions::is_null::IsNull,
        json_client::to_bind_trait::ToBind,
        query_builder::{
            IsOpExpression, ManyExpressions, OpExpression, SqlSyntax,
            functional_expr::ManyBoxedExpressions,
        },
        update_mod::update,
    };

    pub struct Set<S>(String, Box<dyn ToBind<S> + Send>);

    impl<S> OpExpression for Set<S> {}
    impl<S: 'static> Expression<'static, S> for Set<S> {
        fn expression(self, ctx: &mut QueryBuilder<'static, S>)
        where
            S: DatabaseExt,
        {
            ctx.sanitize(&self.0);
            ctx.syntax(&" = ");
            ctx.bind(self.1);
        }
    }

    use super::*;

    impl<S> DynamicCollection<S> {
        pub fn validate_and_cast(
            self,
        ) -> Result<Arc<dyn JsonCollection<S> + Send + Sync>, AddCollectionError>
        where
            S: DatabaseExt,
            DynamicCollection<S>: JsonCollection<S>,
        {
            if self.name.is_empty() || self.name.is_case(Case::Pascal).not() {
                return Err(AddCollectionError::InvalidName(self.name));
            }
            Ok(Arc::new(self))
        }
    }

    impl<S: Database> Collection for DynamicCollection<S> {
        fn table_name(&self) -> &str {
            &self.name
        }
        fn table_name_lower_case(&self) -> &str {
            &self.name_lower_case
        }
        // type PartialInput = JsonValue;
        // type PartialValidationError = DeserializeError;
        type Partial = JsonPartial<S>;
        // fn validate_partial(
        //     &self,
        //     input: Self::PartialInput,
        // ) -> Result<Self::Partial, Self::PartialValidationError> {

        // }

        // type DataInput = JsonValue;
        // type DataValidationError = DeserializeError;
        type Data = JsonData<S>;
        // fn validate_data(
        //     &self,
        //     input: Self::DataInput,
        // ) -> Result<Self::Data, Self::DataValidationError> {

        // }

        type Id = SingleIncremintalInt;

        fn id(&self) -> &Self::Id {
            &SingleIncremintalInt
        }
    }

    impl<S> JsonCollection<S> for DynamicCollection<S>
    where
        S: DatabaseForJsonClient,
        // Vec<Box<dyn ToBind<S> + std::marker::Send>>: ManyExpressions<'static, S>,
        // S: Database<TypeInfo: Type<S>> + DatabaseExt + 'static,
    {
        fn data_to_expression(
            &self,
            data: JsonValue,
        ) -> Result<Box<dyn ManyBoxedExpressions<S> + Send>, DeserializeError> {
            let mut v: Vec<Box<dyn ToBind<S> + Send>> = vec![];

            let mut map = from_value::<HashMap<String, JsonValue>>(data)?;

            for field in self.fields.iter() {
                match (&field.is_optional, map.remove(&field.name)) {
                    (true, None) => v.push(Box::new(())),
                    (false, None) => {
                        return Err(de::Error::custom("cannot find filed with name "));
                    }
                    (_, Some(value)) => {
                        v.push(field.type_info.to_bind(value)?);
                    }
                }
            }

            Ok(Box::new(v))
        }

        fn partial_to_expressions(
            &self,
            data: JsonValue,
        ) -> Result<Box<dyn ManyBoxedExpressions<S> + Send>, DeserializeError> {
            let mut v: Vec<Set<S>> = vec![];

            let mut map = from_value::<HashMap<String, update<JsonValue>>>(data)?;

            for field in self.fields.iter() {
                match map.remove(&field.name) {
                    Some(update::keep) | None => {
                        continue;
                    }
                    Some(update::set(set)) => {
                        let boxxed = field.type_info.to_bind(set)?;
                        v.push(Set(field.name.clone(), boxxed));
                    }
                }
            }

            Ok(Box::new(v))
        }

        fn table_name(&self) -> &str {
            &self.name
        }

        fn table_name_lower_case(&self) -> &str {
            &self.name_lower_case
        }

        fn from_row_two_alias<'r>(&self, row: crate::from_row::two_alias<'r, <S>::Row>) -> JsonValue
        where
            S: sqlx::Database,
        {
            panic!(
                "as far as I know two alias are only used for links. \nrow: {:?}",
                debug_row(row.0)
            )
        }

        fn from_row_pre_alias<'r>(&self, row: crate::from_row::pre_alias<'r, <S>::Row>) -> JsonValue
        where
            S: sqlx::Database,
        {
            let mut obj = serde_json::Map::new();
            for e in self.fields.iter() {
                let jc = e.type_info.from_row(&format!("{}{}", row.1, e.name), row.0);
                obj.insert(e.name.to_string(), jc);
            }
            obj.into()
        }

        fn members(&self) -> Vec<String> {
            self.fields.iter().map(|e| e.name.to_string()).collect()
        }
    }

    #[skip]
    impl JsonCollection<Sqlite> for DynamicCollection {
        fn data_to_expression(
            &self,
            data: JsonValue,
        ) -> Result<Box<dyn ManyBoxedExpressions<Sqlite> + Send>, DeserializeError> {
            todo!()
        }

        fn partial_to_expressions(
            &self,
            partial: JsonValue,
        ) -> Result<Box<dyn ManyBoxedExpressions<Sqlite> + Send>, DeserializeError> {
            todo!()
        }

        fn table_name(&self) -> &str {
            todo!()
        }

        fn table_name_lower_case(&self) -> &str {
            todo!()
        }

        fn from_row_pre_alias<'r>(
            &self,
            row: crate::from_row::pre_alias<'r, SqliteRow>,
        ) -> JsonValue {
            todo!()
        }

        fn members(&self) -> Vec<String> {
            todo!()
        }

        fn from_row_two_alias<'r>(
            &self,
            row: crate::from_row::two_alias<'r, <Sqlite as sqlx::Database>::Row>,
        ) -> JsonValue
        where
            Sqlite: sqlx::Database,
        {
            todo!()
        }
    }

    // impl CollectionBasic for DynamicCollection {
    //     fn table_name(&self) -> &str {
    //         todo!()
    //     }
    //     fn table_name_lower_case(&self) -> &str {
    //         todo!()
    //     }
    // }

    // impl Collection for DynamicCollection {
    //     type Partial = ();
    //     type Data = ();
    //     type Id = SingleIncremintalInt;
    //     fn id(&self) -> &Self::Id {
    //         &SingleIncremintalInt
    //     }
    // }

    // #[allow(unused)]
    // impl<'r, R: Row> FromRowAlias<'r, R> for DynamicCollection {
    //     type FromRowData = ();

    //     fn no_alias(&self, row: &'r R) -> Result<Self::FromRowData, crate::from_row::FromRowError> {
    //         todo!()
    //     }

    //     fn pre_alias(
    //         &self,
    //         row: crate::from_row::pre_alias<'r, R>,
    //     ) -> Result<Self::FromRowData, crate::from_row::FromRowError>
    //     where
    //         R: Row,
    //     {
    //         todo!()
    //     }

    //     fn post_alias(
    //         &self,
    //         row: crate::from_row::post_alias<'r, R>,
    //     ) -> Result<Self::FromRowData, crate::from_row::FromRowError>
    //     where
    //         R: Row,
    //     {
    //         todo!()
    //     }

    //     fn two_alias(
    //         &self,
    //         row: crate::from_row::two_alias<'r, R>,
    //     ) -> Result<Self::FromRowData, crate::from_row::FromRowError>
    //     where
    //         R: Row,
    //     {
    //         todo!()
    //     }
    // }

    // impl<S> Members<S> for DynamicCollection
    // where
    //     S: DatabaseExt,
    // {
    //     fn members_names(&self) -> Vec<String> {
    //         todo!()
    //     }
    // }
}
