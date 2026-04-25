pub mod to_bind_trait {
    use crate::expressions::is_null::IsNull as IsNullTrait;
    use sqlx::Database;
    use sqlx::Encode;
    use sqlx::Type;
    use sqlx::encode::IsNull as IsNullEnum;
    use sqlx::encode::IsNull;
    use sqlx::error::BoxDynError;

    pub trait ToBind<S: Database>: Send {
        fn clone_to_box<'q>(&self) -> Box<dyn ToBind<S> + Send + 'q>;
        fn bind_ref<'q>(&self, buf: &mut S::ArgumentBuffer<'q>) -> Result<IsNull, BoxDynError>;
        fn bind_boxed<'q>(
            self: Box<Self>,
            buf: &mut <S as sqlx::Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, BoxDynError>;
        fn is_null(&self) -> bool;
    }

    impl<S: Database> ToBind<S> for () {
        fn clone_to_box<'q>(&self) -> Box<dyn ToBind<S> + Send + 'q> {
            Box::new(())
        }
        fn bind_ref<'q>(
            &self,
            _: &mut <S as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNullEnum, BoxDynError> {
            Ok(IsNullEnum::Yes)
        }

        fn bind_boxed<'q>(
            self: Box<Self>,
            _: &mut <S as sqlx::Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNullEnum, BoxDynError> {
            Ok(IsNullEnum::Yes)
        }

        fn is_null(&self) -> bool {
            true
        }
    }

    impl<S, T> ToBind<S> for T
    where
        T: Clone,
        T: IsNullTrait,
        T: Send,
        S: Database,
        T: for<'q> Encode<'q, S> + Type<S> + 'static,
    {
        fn clone_to_box<'q>(&self) -> Box<dyn ToBind<S> + Send + 'q> {
            Box::new(self.clone())
        }
        fn is_null(&self) -> bool {
            T::is_null()
        }
        fn bind_ref<'q>(
            &self,
            buf: &mut <S as sqlx::Database>::ArgumentBuffer<'q>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            Encode::encode_by_ref(self, buf)
        }
        fn bind_boxed<'q>(
            self: Box<Self>,
            buf: &mut <S as sqlx::Database>::ArgumentBuffer<'q>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            Encode::encode(*self, buf)
        }
    }

    impl<S: Database> Clone for Box<dyn ToBind<S> + Send> {
        fn clone(&self) -> Self {
            self.clone_to_box()
        }
    }

    impl<S> Type<S> for Box<dyn ToBind<S> + Send>
    where
        S: sqlx::Database,
        // S: DatabaseForJsonClient,
    {
        fn type_info() -> <S as Database>::TypeInfo {
            todo!()
            // S::type_info_default()
        }
    }

    impl<'q, S> Encode<'q, S> for Box<dyn ToBind<S> + Send>
    where
        S: Database,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <S as sqlx::Database>::ArgumentBuffer<'q>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            self.bind_ref(buf)
        }
        fn encode(
            self,
            buf: &mut <S as Database>::ArgumentBuffer<'q>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
        where
            Self: Sized,
        {
            self.bind_boxed(buf)
        }
    }

    mod expression_impls {
        use super::*;
        use crate::database_extention::DatabaseExt;
        use crate::query_builder::Expression;
        use crate::query_builder::OpExpression;
        use crate::query_builder::StatementBuilder;
        use sqlx::Database;

        impl<S> OpExpression for Box<dyn ToBind<S> + Send> {}
        impl<'q, S> Expression<'q, S> for Box<dyn ToBind<S> + Send>
        where
            S: Database + DatabaseExt,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>)
            where
                S: DatabaseExt,
            {
                if self.is_null() {
                    ctx.syntax(&"NULL");
                } else {
                    ctx.bind(self);
                }
            }
        }
    }
}

pub mod sqlx_type_ident {
    use super::to_bind_trait::ToBind;
    use crate::database_extention::DatabaseExt;
    use crate::from_row::FromRowError;
    use crate::from_row::RowPreAliased;
    use crate::from_row::RowTwoAliased;
    use crate::query_builder::StatementBuilder;
    use crate::query_builder::SyntaxAsType;
    use crate::query_builder::functional_expr::BoxedExpression;
    use core::fmt;
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use serde_json::Error as DeserializeError;
    use serde_json::Value as JsonValue;
    use serde_json::from_value;
    use serde_json::json;
    use serde_json::to_value;
    use sqlx::ColumnIndex;
    use sqlx::Database;
    use sqlx::Decode;
    use std::marker::PhantomData;

    pub trait SqlxTypeHandler<S>: Send + Sync {
        fn clone_self(&self) -> Box<dyn SqlxTypeHandler<S> + Send + Sync>;
        fn type_name(&self) -> &str;
        fn type_expression(&self) -> Box<dyn BoxedExpression<S> + Send>;
        fn to_bind<'q>(
            &self,
            value: JsonValue,
        ) -> Result<Box<dyn ToBind<S> + Send>, DeserializeError>;
        fn from_row_two_alias(
            &self,
            is_optional: bool,
            name: &str,
            row: RowTwoAliased<'_, S::Row>,
        ) -> Result<JsonValue, FromRowError>
        where
            S: Database;
        fn from_row_pre_alias(
            &self,
            is_optional: bool,
            name: &str,
            row: RowPreAliased<'_, S::Row>,
        ) -> Result<JsonValue, FromRowError>
        where
            S: Database;
        fn from_row_no_alias(
            &self,
            is_optional: bool,
            name: &str,
            row: &S::Row,
        ) -> Result<JsonValue, FromRowError>
        where
            S: Database;
    }

    impl<S> Clone for Box<dyn SqlxTypeHandler<S> + Send + Sync> {
        fn clone(&self) -> Self {
            self.clone_self()
        }
    }

    impl<'q, S: DatabaseExt + 'q> fmt::Debug for Box<dyn SqlxTypeHandler<S> + Send + Sync + 'q> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = StatementBuilder::new(self.type_expression()).unwrap().0;
            let t = self.type_name();
            write!(f, "Type {{ sql_expr: {s}, rust_type: {t} }}")
        }
    }

    impl<S, T> SqlxTypeHandler<S> for PhantomData<T>
    where
        S: DatabaseExt,
        T: Send + Sync + 'static,
        T: ToBind<S>,
        T: DeserializeOwned + Serialize,
        T: 'static + for<'d> Decode<'d, S> + sqlx::Type<S>,
        for<'a> &'a str: ColumnIndex<S::Row>,
    {
        fn type_name(&self) -> &str {
            std::any::type_name::<T>()
        }
        fn to_bind(&self, value: JsonValue) -> Result<Box<dyn ToBind<S> + Send>, DeserializeError> {
            let s: T = from_value(value).expect("bug: claw_ql should clear out");
            Ok(Box::new(s))
        }
        fn type_expression(&self) -> Box<dyn BoxedExpression<S> + Send> {
            Box::new(SyntaxAsType::<T>(PhantomData))
        }
        fn clone_self(&self) -> Box<dyn SqlxTypeHandler<S> + Send + Sync> {
            Box::new(PhantomData::<T>)
        }
        fn from_row_no_alias(
            &self,
            i: bool,
            name: &str,
            row: &<S>::Row,
        ) -> Result<JsonValue, FromRowError>
        where
            S: Database,
        {
            let s: Option<T> = sqlx::Row::try_get(row, name)?;

            let s = match (s, i) {
                (None, true) => return Ok(json!(null)),
                (None, false) => {
                    panic!("claw_ql_bug: value is non-null and was assumed to be null")
                }
                (Some(e), _) => e,
            };

            Ok(to_value(s).expect("claw_ql_bug: when serialize ever fail?"))
        }
        fn from_row_two_alias(
            &self,
            is_optional: bool,
            name: &str,
            row: RowTwoAliased<'_, S::Row>,
        ) -> Result<JsonValue, FromRowError>
        where
            S: Database,
        {
            let s: Option<T> = row.try_get(name)?;

            let s = match (s, is_optional) {
                (None, true) => return Ok(json!(null)),
                (None, false) => {
                    panic!("claw_ql_bug: value is non-null and was assumed to be null")
                }
                (Some(e), _) => e,
            };

            Ok(to_value(s).expect("claw_ql_bug: when serialize ever fail?"))
        }
        fn from_row_pre_alias(
            &self,
            is_optional: bool,
            name: &str,
            row: RowPreAliased<'_, S::Row>,
        ) -> Result<JsonValue, FromRowError>
        where
            S: Database,
        {
            let s: Option<T> = row.try_get(name)?;

            let s = match (s, is_optional) {
                (None, true) => return Ok(json!(null)),
                (None, false) => {
                    panic!("claw_ql_bug: value is non-null and was assumed to be null")
                }
                (Some(e), _) => e,
            };

            Ok(to_value(s).expect("claw_ql_bug: when serialize ever fail?"))
        }
    }
}

pub mod database_for_json_client {
    use std::marker::PhantomData;

    use sqlx::{ColumnIndex, Decode, Encode, Type};

    use crate::{
        collections::SingleIncremintalInt,
        database_extention::DatabaseExt,
        extentions::common_expressions::MigrateExpression,
        fix_executor::ExecutorTrait,
        query_builder::{Expression, StatementBuilder},
    };

    use super::sqlx_type_ident::SqlxTypeHandler;

    pub trait DatabaseForJsonClient: DatabaseExt + ExecutorTrait + Send + 'static {
        fn support_string() -> Box<dyn SqlxTypeHandler<Self> + Send + Sync>;
        fn support_boolean() -> Box<dyn SqlxTypeHandler<Self> + Send + Sync>;
        fn support_int() -> Box<dyn SqlxTypeHandler<Self> + Send + Sync>;
        fn id_migrate_expression(ctx: &mut StatementBuilder<'static, Self>);
    }

    impl<S> DatabaseForJsonClient for S
    where
        S: DatabaseExt + Send,
        S: ExecutorTrait,
        for<'s> &'s str: ColumnIndex<S::Row>,
        String: Type<S> + for<'q> Encode<'q, S> + for<'q> Decode<'q, S>,
        bool: Type<S> + for<'q> Encode<'q, S> + for<'q> Decode<'q, S>,
        i64: Type<S> + for<'q> Encode<'q, S> + for<'q> Decode<'q, S>,
        SingleIncremintalInt<&'static str>:
            MigrateExpression<MigrateExpression: for<'q> Expression<'q, S>>,
    {
        fn support_string() -> Box<dyn SqlxTypeHandler<Self> + Send + Sync> {
            Box::new(PhantomData::<String>)
        }

        fn support_boolean() -> Box<dyn SqlxTypeHandler<Self> + Send + Sync> {
            Box::new(PhantomData::<bool>)
        }

        fn support_int() -> Box<dyn SqlxTypeHandler<Self> + Send + Sync> {
            Box::new(PhantomData::<i64>)
        }

        fn id_migrate_expression(ctx: &mut StatementBuilder<'static, Self>) {
            SingleIncremintalInt("id")
                .migrate_expression()
                .expression(ctx);
        }
    }
}

pub mod json_client {
    use crate::json_client::dynamic_collection::DynamicCollection;
    use sqlx::{Database, Pool};
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use tokio::sync::RwLock as TrwLock;

    type Jc<S> = Arc<TrwLock<DynamicCollection<S>>>;

    pub struct JsonClientOption {
        pub check_for_unique_filters_on_update: bool,
    }

    impl JsonClientOption {
        pub fn default_setting() -> Self {
            JsonClientOption {
                check_for_unique_filters_on_update: true,
            }
        }
    }

    /// JsonClient suitable for using in backends
    ///
    /// heavely relies on runtime-friendly extentions traits to this crate's generic traits
    /// so there is no cost to use this if you implement those base traits
    ///
    /// allows you to execute notable `impl Operation` types like `FetchOne`, `InsertOne`, `UpdateOne`, and `DeleteOne`,
    /// along with other mutable operations like 'add_collection', 'drop_collection', 'modify_collection'.
    ///
    /// I'm planning to make "DynamicJsonClient" that provides more mutable operations to extend types, links, and errors, but this is good enough for now.
    pub struct JsonClient<S: Database> {
        pub collections: HashMap<String, Jc<S>>,
        pub migrations: Vec<String>,
        pub options: JsonClientOption,
        pub pool: Pool<S>,
        pub links: LinkInformations,
    }

    #[derive(Default, Debug)]
    pub struct LinkInformations {
        pub optional_to_many: HashSet<(String, String)>,
        pub timestamped: HashSet<String>,
    }

    impl<S: Database> From<Pool<S>> for JsonClient<S> {
        fn from(pool: Pool<S>) -> Self {
            Self {
                collections: Default::default(),
                migrations: Default::default(),
                options: JsonClientOption::default_setting(),
                pool,
                links: LinkInformations::default(),
            }
        }
    }
}

pub mod supported_filter {
    use crate::database_extention::DatabaseExt;
    use crate::expressions::col_eq;
    use crate::json_client::dynamic_collection::DynamicCollection;
    use crate::query_builder::functional_expr::BoxedExpression;
    use serde::{Deserialize, Serialize};
    use serde_json::Value as JsonValue;

    #[derive(Deserialize, Clone, Debug)]
    #[non_exhaustive]
    pub enum SupportedFilter {
        ColEq(col_eq<String, JsonValue>),
    }

    #[derive(Debug, Serialize)]
    pub enum InvalidFilter {
        FieldNotFound(String),
        TypeMismatch(String),
    }

    pub fn parse_supported_filter<'q, S>(
        input: Vec<SupportedFilter>,
        base: &DynamicCollection<S>,
    ) -> Result<Vec<Box<dyn BoxedExpression<S> + Send>>, InvalidFilter>
    where
        S: DatabaseExt,
    {
        let mut ret: Vec<Box<dyn BoxedExpression<S> + Send>> = vec![];
        for each in input {
            match each {
                SupportedFilter::ColEq(col_eq { col, eq }) => {
                    if let Some(o) = base.fields.iter().find(|f| f.name == col) {
                        if let Ok(s) = o.type_info.to_bind(eq) {
                            ret.push(Box::new(col_eq { col, eq: s }));
                        } else {
                            return Err(InvalidFilter::TypeMismatch(col));
                        }
                    } else {
                        return Err(InvalidFilter::FieldNotFound(col));
                    };
                }
            }
        }

        Ok(ret)
    }
}

pub mod links_utils {
    use std::ops::Not;

    use sqlx::Database;

    use crate::{
        json_client::{dynamic_collection::DynamicCollection, json_client::JsonClient},
        links::{DefaultRelationKey, relation_optional_to_many::OptionalToMany},
    };

    pub async fn get_optional_to_many<S: Database>(
        base: &DynamicCollection<S>,
        to: String,
        jc: &JsonClient<S>,
    ) -> Result<OptionalToMany<DefaultRelationKey, DynamicCollection<S>, DynamicCollection<S>>, ()>
    {
        if jc
            .links
            .optional_to_many
            .contains(&(base.name_lower_case.clone(), to.to_string()))
            .not()
        {
            panic!()
        }

        let to = if let Some(e) = jc.collections.get(to.as_str()) {
            e.read().await.clone()
        } else {
            panic!();
        };

        Ok(OptionalToMany {
            foriegn_key: DefaultRelationKey,
            from: base.clone(),
            to,
        })
    }
}

pub mod dynamic_collection {
    use core::fmt;

    use serde::Deserialize;

    use crate::{database_extention::DatabaseExt, json_client::sqlx_type_ident::SqlxTypeHandler};

    pub struct DynamicCollection<S> {
        pub name: String,
        pub name_lower_case: String,
        pub fields: Vec<DynamicField<Box<dyn SqlxTypeHandler<S> + Send + Sync>>>,
    }

    impl<S> fmt::Debug for DynamicCollection<S>
    where
        S: DatabaseExt,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "DynamicCollection {{ name: {}, fields: {:?}, db: {} }}",
                self.name,
                self.fields,
                S::NAME
            )
        }
    }

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

    #[derive(Debug, Clone, Deserialize)]
    pub struct DynamicField<T> {
        pub name: String,
        pub is_optional: bool,
        pub type_info: T,
    }

    mod expression {
        use crate::{
            database_extention::DatabaseExt,
            json_client::{dynamic_collection::DynamicField, sqlx_type_ident::SqlxTypeHandler},
            query_builder::{Expression, OpExpression, StatementBuilder},
        };

        impl<'bx, S> OpExpression for DynamicField<Box<dyn SqlxTypeHandler<S> + Send + Sync + 'bx>> {}

        impl<'q, S: DatabaseExt> Expression<'q, S>
            for DynamicField<Box<dyn SqlxTypeHandler<S> + Send + Sync>>
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
                ctx.sanitize(&self.name);
            }
        }
    }

    mod from_row {
        use sqlx::Database;
        use sqlx::Row;

        use super::DynamicField;
        use crate::from_row::FromRowAlias;
        use crate::from_row::FromRowData;
        use crate::from_row::FromRowError;
        use crate::from_row::RowPostAliased;
        use crate::from_row::RowPreAliased;
        use crate::from_row::RowTwoAliased;
        use crate::json_client::sqlx_type_ident::SqlxTypeHandler;

        impl<'bx, S> FromRowData for Vec<DynamicField<Box<dyn SqlxTypeHandler<S> + Send + Sync + 'bx>>> {
            type RData = serde_json::Map<String, serde_json::Value>;
        }

        impl<'bx, 'r, S> FromRowAlias<'r, S::Row>
            for Vec<DynamicField<Box<dyn SqlxTypeHandler<S> + Send + Sync + 'bx>>>
        where
            S: Database,
        {
            fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
                let mut oj = serde_json::Map::new();
                for field in self.iter() {
                    let s =
                        field
                            .type_info
                            .from_row_no_alias(field.is_optional, &field.name, row)?;
                    oj.insert(field.name.clone(), s);
                }
                Ok(oj)
            }

            fn pre_alias(&self, row: RowPreAliased<'r, S::Row>) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                let mut oj = serde_json::Map::new();
                for field in self.iter() {
                    let s = field.type_info.from_row_pre_alias(
                        field.is_optional,
                        &field.name,
                        row.clone(),
                    )?;
                    oj.insert(field.name.clone(), s);
                }
                Ok(oj)
            }

            fn post_alias(
                &self,
                row: RowPostAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                let _ = row;
                panic!("in the process of deprecating this method");
            }

            fn two_alias(&self, row: RowTwoAliased<'r, S::Row>) -> Result<Self::RData, FromRowError>
            where
                S::Row: Row,
            {
                let mut oj = serde_json::Map::new();
                for field in self.iter() {
                    let s = field.type_info.from_row_two_alias(
                        field.is_optional,
                        &field.name,
                        row.clone(),
                    )?;
                    oj.insert(field.name.clone(), s);
                }
                Ok(oj)
            }
        }

        impl<S> FromRowData for DynamicField<Box<dyn SqlxTypeHandler<S> + Send + Sync>> {
            type RData = (String, serde_json::Value);
        }

        impl<'r, S> FromRowAlias<'r, S::Row> for DynamicField<Box<dyn SqlxTypeHandler<S> + Send + Sync>>
        where
            S: Database,
        {
            fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
                Ok((
                    self.name.clone(),
                    self.type_info.from_row_no_alias(false, &self.name, row)?,
                ))
            }
            fn pre_alias(
                &self,
                row: RowPreAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError> {
                Ok((
                    self.name.clone(),
                    self.type_info.from_row_pre_alias(false, &self.name, row)?,
                ))
            }
            fn post_alias(
                &self,
                _: RowPostAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError> {
                panic!("in the process of deprecating this method")
            }
            fn two_alias(
                &self,
                row: RowTwoAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError> {
                Ok((
                    self.name.clone(),
                    self.type_info.from_row_two_alias(false, &self.name, row)?,
                ))
            }
        }
    }

    mod impl_on_migrate {
        use std::ops::Not;

        use super::DynamicCollection;
        use crate::{
            json_client::{
                database_for_json_client::DatabaseForJsonClient, dynamic_collection::DynamicField,
            },
            on_migrate::OnMigrate,
            query_builder::{
                Expression, OpExpression, StatementBuilder,
                essential_syntax::{CLOSE_PARANTHESIS, OPEN_PARANTHESIS},
                functional_expr::BoxedExpression,
            },
        };

        pub struct MigrateDynamicCollection<S> {
            pub name: String,
            pub fields: Vec<DynamicField<Box<dyn BoxedExpression<S> + Send>>>,
        }

        impl<S> OnMigrate for DynamicCollection<S> {
            type Statements = MigrateDynamicCollection<S>;
            fn statments(&self) -> Self::Statements {
                MigrateDynamicCollection {
                    name: self.name.clone(),
                    fields: self
                        .fields
                        .iter()
                        .map(|e| DynamicField {
                            name: e.name.clone(),
                            is_optional: e.is_optional,
                            type_info: e.type_info.type_expression(),
                        })
                        .collect(),
                }
            }
        }

        impl<S> OpExpression for MigrateDynamicCollection<S> {}

        impl<S> Expression<'static, S> for MigrateDynamicCollection<S>
        where
            S: DatabaseForJsonClient,
        {
            fn expression(self, ctx: &mut StatementBuilder<'static, S>) {
                ctx.syntax("CREATE TABLE ");
                ctx.sanitize(self.name.as_str());
                ctx.syntax(" ");
                ctx.syntax(OPEN_PARANTHESIS);

                S::id_migrate_expression(ctx);

                for field in self.fields.into_iter() {
                    ctx.syntax(&", ");
                    ctx.sanitize(field.name.as_str());
                    ctx.syntax(" ");
                    field.type_info.expression(ctx);
                    if field.is_optional.not() {
                        ctx.syntax(&" NOT NULL");
                    }
                }
                ctx.syntax(CLOSE_PARANTHESIS);
                ctx.syntax(";");
            }
        }
    }

    mod collection_impls {
        use crate::{
            collections::{Collection, SingleIncremintalInt},
            json_client::dynamic_collection::DynamicCollection,
        };

        impl<S> Collection for DynamicCollection<S> {
            fn table_name(&self) -> &str {
                &self.name
            }
            fn table_name_lower_case(&self) -> &str {
                &self.name_lower_case
            }

            type Data = serde_json::Value;

            type Id = SingleIncremintalInt<String>;

            fn id(&self) -> Self::Id {
                SingleIncremintalInt(self.name.clone())
            }
        }
    }

    mod str_aliased_impls {
        use std::ops::Not;

        use crate::{
            extentions::common_expressions::Aliased,
            query_builder::{IsOpExpression, ManyExpressions},
        };

        use super::DynamicCollection;

        pub struct DynamicAliasedCols {
            pub table: String,
            pub cols: Vec<String>,
            pub num: Option<usize>,
            pub alias: &'static str,
        }

        impl<S> Aliased for DynamicCollection<S> {
            type Aliased = DynamicAliasedCols;

            fn aliased(&self, alias: &'static str) -> Self::Aliased {
                DynamicAliasedCols {
                    table: self.name.clone(),
                    cols: self.fields.iter().map(|e| e.name.clone()).collect(),
                    num: None,
                    alias,
                }
            }
            type NumAliased = DynamicAliasedCols;
            fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
                DynamicAliasedCols {
                    table: self.name.clone(),
                    cols: self.fields.iter().map(|e| e.name.clone()).collect(),
                    num: Some(num),
                    alias,
                }
            }
        }

        impl IsOpExpression for DynamicAliasedCols {
            fn is_op(&self) -> bool {
                self.cols.is_empty().not()
            }
        }
        impl<'q, S> ManyExpressions<'q, S> for DynamicAliasedCols {
            fn expression(
                self,
                start: &'static str,
                join: &'static str,
                ctx: &mut crate::query_builder::StatementBuilder<'q, S>,
            ) where
                S: crate::database_extention::DatabaseExt,
            {
                let len = self.cols.len();
                if len == 0 {
                    return;
                }
                ctx.syntax(start);
                for (i, col) in self.cols.into_iter().enumerate() {
                    ctx.sanitize(&self.table);
                    ctx.syntax(".");
                    ctx.sanitize(&col);
                    ctx.syntax(" AS ");
                    if let Some(num) = self.num {
                        ctx.sanitize_strings((self.alias, num, col.as_str()));
                    } else {
                        ctx.sanitize_strings((self.alias, col.as_str()));
                    }
                    if i < len - 1 {
                        ctx.syntax(join);
                    }
                }
            }
        }
    }

    mod expression_table_name {
        use crate::extentions::common_expressions::TableNameExpression;

        use super::DynamicCollection;

        impl<S> TableNameExpression for DynamicCollection<S> {
            type TableNameExpression = String;
            fn table_name_expression(&self) -> Self::TableNameExpression {
                self.name.clone()
            }
        }
    }

    mod from_row_impls {
        use crate::{
            database_extention::DatabaseExt,
            from_row::{FromRowAlias, FromRowData},
            json_client::{
                database_for_json_client::DatabaseForJsonClient,
                dynamic_collection::DynamicCollection,
            },
        };
        use serde_json::Value as JsonValue;
        use sqlx::ColumnIndex;

        impl<S> FromRowData for DynamicCollection<S> {
            type RData = JsonValue;
        }

        impl<'r, S> FromRowAlias<'r, S::Row> for DynamicCollection<S>
        where
            S: DatabaseExt,
            for<'s> &'s str: ColumnIndex<S::Row>,
        {
            fn no_alias(
                &self,
                row: &'r S::Row,
            ) -> Result<Self::RData, crate::from_row::FromRowError> {
                let mut oj = serde_json::Map::new();
                for field in self.fields.iter() {
                    let s =
                        field
                            .type_info
                            .from_row_no_alias(field.is_optional, &field.name, row)?;
                    oj.insert(field.name.to_string(), s);
                }
                Ok(JsonValue::Object(oj))
            }

            fn pre_alias(
                &self,
                row: crate::from_row::RowPreAliased<'r, S::Row>,
            ) -> Result<Self::RData, crate::from_row::FromRowError>
            where
                S::Row: sqlx::Row,
            {
                let mut oj = serde_json::Map::new();

                for field in self.fields.iter() {
                    let s = field.type_info.from_row_pre_alias(
                        field.is_optional,
                        &field.name,
                        row.clone(),
                    )?;
                    oj.insert(field.name.to_string(), s);
                }

                Ok(JsonValue::Object(oj))
            }

            fn post_alias(
                &self,
                _: crate::from_row::RowPostAliased<'r, S::Row>,
            ) -> Result<Self::RData, crate::from_row::FromRowError>
            where
                S::Row: sqlx::Row,
            {
                todo!("do I need to even implement this function")
            }

            fn two_alias(
                &self,
                row: crate::from_row::RowTwoAliased<'r, S::Row>,
            ) -> Result<Self::RData, crate::from_row::FromRowError>
            where
                S::Row: sqlx::Row,
            {
                let mut oj = serde_json::Map::new();

                // panic!("debug_row: {:?}", DebugRow(row.0));

                for field in self.fields.iter() {
                    let s = field.type_info.from_row_two_alias(
                        field.is_optional,
                        &field.name,
                        row.clone(),
                    )?;
                    oj.insert(field.name.to_string(), s);
                }

                Ok(JsonValue::Object(oj))
            }
        }
    }

    #[claw_ql_macros::skip]
    mod expression_on_update {
        use super::*;
        use crate::database_extention::DatabaseExt;
        use crate::extentions::common_expressions::OnUpdate;
        use crate::json_client::to_bind_trait::ToBind;
        use crate::query_builder::IsOpExpression;
        use crate::query_builder::ManyExpressions;
        use crate::query_builder::StatementBuilder;
        use serde_json::Value as JsonValue;
        use std::ops::Not;

        pub struct ToBindUpdateMany<'q, S> {
            pub vec: Vec<(String, Box<dyn ToBind<'q, S> + Send + 'q>)>,
        }

        impl<'q, S> IsOpExpression for ToBindUpdateMany<'q, S> {
            fn is_op(&self) -> bool {
                self.vec.is_empty().not()
            }
        }
        impl<'q, S: 'q> ManyExpressions<'q, S> for ToBindUpdateMany<'q, S> {
            fn expression(
                mut self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'q, S>,
            ) where
                S: DatabaseExt,
            {
                if self.vec.is_empty() {
                    return;
                }

                ctx.syntax(start);
                let last = self.vec.pop();

                for each in self.vec {
                    ctx.sanitize(each.0.as_str());
                    ctx.syntax(&" = ");
                    ctx.bind(each.1);
                    ctx.syntax(join);
                }

                if let Some(value) = last {
                    ctx.sanitize(value.0.as_str());
                    ctx.syntax(&" = ");
                    ctx.bind(value.1);
                }
            }
        }

        impl<'q, S> OnUpdate for DynamicCollection<S> {
            type UpdateInput = JsonValue;
            type UpdateExpression = ToBindUpdateMany<'q, S>;
            fn validate_on_update(&self, input: Self::UpdateInput) -> Self::UpdateExpression {
                let _ = input;
                todo!("refactor to infallible")
                // let mut v: Vec<(String, Box<dyn ToBind<S> + Send>)> = vec![];

                // let mut map = from_value::<HashMap<String, Update<JsonValue>>>(input)?;

                // for field in self.fields.iter() {
                //     match map.remove(&field.name) {
                //         Some(Update::Keep) | None => {
                //             continue;
                //         }
                //         Some(Update::Set(set)) => {
                //             let boxxed = field.type_info.to_bind(set)?;
                //             v.push((field.name.clone(), boxxed));
                //         }
                //     }
                // }

                // if map.is_empty().not() {
                //     return Err(de::Error::custom(format!(
                //         "cannot find filed with name {:?}",
                //         map.keys().collect::<Vec<_>>()
                //     )));
                // }

                // Ok(ToBindUpdateMany { vec: v })
            }
        }
    }

    #[claw_ql_macros::skip]
    mod expression_on_insert {
        use std::ops::Not;

        use crate::database_extention::DatabaseExt;
        use crate::json_client::to_bind_trait::ToBind;
        use crate::query_builder::IsOpExpression;
        use crate::query_builder::ManyExpressions;
        use crate::query_builder::StatementBuilder;
        use sqlx::Database;

        pub struct ToBindSetMany<S> {
            pub vec: Vec<Box<dyn ToBind<S> + Send>>,
        }

        impl<S> IsOpExpression for ToBindSetMany<S> {
            fn is_op(&self) -> bool {
                self.vec.is_empty().not()
            }
        }

        impl<S: 'static> ManyExpressions<'static, S> for ToBindSetMany<S> {
            fn expression(
                mut self,
                start: &'static str,
                join: &'static str,
                ctx: &mut StatementBuilder<'static, S>,
            ) where
                S: DatabaseExt,
            {
                if self.vec.is_empty() {
                    return;
                }

                ctx.syntax(start);
                let last = self.vec.pop();

                for each in self.vec {
                    ctx.bind(each);
                    ctx.syntax(join);
                }

                if let Some(value) = last {
                    ctx.bind(value);
                }
            }
        }

        #[claw_ql_macros::skip]
        impl<S> OnInsert for DynamicCollection<S>
        where
            S: Database,
        {
            type InsertInput = serde_json::Value;
            // type InsertError = DeserializeError;
            type InsertExpression = ToBindSetMany<S>;
            fn validate_on_insert(&self, input: Self::InsertInput) -> Self::InsertExpression {
                let _ = input;
                todo!("refactor to infallible")
                // let mut v: Vec<Box<dyn ToBind<S> + Send>> = vec![];

                // let mut map = serde_json::from_value::<HashMap<String, serde_json::Value>>(input)?;

                // for field in self.fields.iter() {
                //     match (&field.is_optional, map.remove(&field.name)) {
                //         (true, None) => v.push(Box::new(()) as Box<dyn ToBind<S> + Send>),
                //         (false, None) => {
                //             return Err(de::Error::custom("cannot find filed with name "));
                //         }
                //         (_, Some(value)) => {
                //             v.push(field.type_info.to_bind(value)?);
                //         }
                //     }
                // }

                // Ok(ToBindSetMany { vec: v })
            }
        }
    }
}

pub mod add_collection {
    use std::ops::Not;
    use std::sync::Arc;

    use crate::json_client::database_for_json_client::DatabaseForJsonClient;
    use crate::json_client::dynamic_collection::{DynamicCollection, DynamicField};
    use crate::json_client::json_client::JsonClient;
    use crate::json_client::sqlx_type_ident::SqlxTypeHandler;
    use crate::on_migrate::OnMigrate;
    use crate::query_builder::StatementBuilder;
    use convert_case::{Case, Casing};
    use serde::{Deserialize, Serialize};
    use sqlx::{Executor, IntoArguments};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub enum TypeSpec {
        String,
        Boolean,
    }

    #[derive(Debug, Deserialize)]
    pub struct AddCollectionInput {
        pub name: String,
        pub fields: Vec<DynamicField<TypeSpec>>,
    }

    pub type AddCollectionOutput = ();

    impl<S> JsonClient<S>
    where
        S: DatabaseForJsonClient,
        for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
        for<'q> <S as sqlx::Database>::Arguments<'q>: IntoArguments<'q, S>,
    {
        pub fn add_collection(
            &mut self,
            input: AddCollectionInput,
        ) -> impl Future<Output = Result<AddCollectionOutput, String>> + Send {
            async move {
                let name = input.name.clone();

                if input.name.is_empty()
                    || input.name.is_case(Case::Snake).not()
                    || input.name.starts_with("ct_")
                    || input.name.starts_with("meta_")
                {
                    Err(format!(
                        "{} should not start with either ct_ or meta_",
                        input.name
                    ))?;
                }

                if input.fields.iter().any(|e| e.name.starts_with("id")) {
                    todo!()
                    // return Err(AddCollectionError::NameViolateId);
                }
                if input.fields.iter().any(|e| e.name.starts_with("fk_")) {
                    todo!()
                    // return Err(AddCollectionError::NameViolateLinks);
                }
                let dc = DynamicCollection {
                    name: input.name.to_case(Case::Pascal),
                    name_lower_case: input.name,
                    fields: input
                        .fields
                        .into_iter()
                        .map(|e| {
                            let type_info: Box<dyn SqlxTypeHandler<S> + Send + Sync> =
                                match e.type_info {
                                    TypeSpec::String => S::support_string(),
                                    TypeSpec::Boolean => S::support_boolean(),
                                };
                            DynamicField {
                                name: e.name,
                                is_optional: e.is_optional,
                                type_info,
                            }
                        })
                        .collect(),
                };

                let mig = StatementBuilder::<S>::new_no_data(dc.clone().statments())
                    .expect("bug: migrations should not have any aditional data");

                sqlx::query(&mig).execute(&self.pool).await.unwrap();

                self.migrations.push(mig);

                if self.collections.get(&name).is_some() {
                    todo!()
                    // return Err(AddCollectionError::CollectionAlreadyExists);
                }

                match self
                    .collections
                    .insert(name, Arc::new(tokio::sync::RwLock::new(dc)))
                {
                    Some(_) => panic!("bug: should not replace old collections"),
                    None => Ok(()),
                }
            }
        }
    }
}

pub mod add_link {
    use serde::Deserialize;
    use sqlx::{Executor, IntoArguments};

    use crate::{
        json_client::{database_for_json_client::DatabaseForJsonClient, json_client::JsonClient},
        links::{DefaultRelationKey, relation_optional_to_many::OptionalToMany},
        on_migrate::OnMigrate,
        query_builder::StatementBuilder,
    };

    #[derive(Debug, Deserialize)]
    #[serde(tag = "ty")]
    pub enum AddLinkInput {
        #[serde(rename = "optional_to_many")]
        OptionalToMany { from: String, to: String },
        #[serde(rename = "timestamp")]
        Timestamp { collection: String },
    }
    pub type AddLinkOutput = ();

    #[derive(Debug)]
    pub enum AddLinkError {
        CollectionDoesntExist(String),
        LinkExist(AddLinkInput),
    }

    #[claw_ql_macros::skip]
    impl<S> JsonClient<S>
    where
        S: DatabaseForJsonClient,
        for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
        for<'q> S::Arguments<'q>: IntoArguments<'q, S>,
    {
        pub fn add_link(
            &mut self,
            input: AddLinkInput,
        ) -> impl Future<Output = Result<AddLinkOutput, AddLinkError>> + Send {
            async move {
                match input {
                    AddLinkInput::Timestamp { collection } => {
                        let collection_lock = self
                            .collections
                            .get(collection.as_str())
                            .ok_or(AddLinkError::CollectionDoesntExist(collection.clone()))?
                            .read()
                            .await;

                        let collection_obj = collection_lock.clone();

                        todo!("perform migration");

                        self.links.timestamped.insert(collection.clone());
                        todo!()
                    }
                    AddLinkInput::OptionalToMany { from, to } => {
                        let from_collection = self
                            .collections
                            .get(from.as_str())
                            .ok_or(AddLinkError::CollectionDoesntExist(from.clone()))?
                            .read()
                            .await
                            .clone();
                        let to_collection = self
                            .collections
                            .get(to.as_str())
                            .ok_or(AddLinkError::CollectionDoesntExist(to.clone()))?
                            .read()
                            .await
                            .clone();

                        let s =
                            StatementBuilder::new_no_data(OnMigrate::statments(&OptionalToMany {
                                foriegn_key: DefaultRelationKey,
                                from: from_collection,
                                to: to_collection,
                            }))
                            .expect("bug: migrations should not have any aditional data");

                        sqlx::query(s.as_str()).execute(&self.pool).await.unwrap();

                        // what if server crash here, this is not atomic op

                        self.migrations.push(s.to_string());

                        self.links
                            .optional_to_many
                            .insert((from.clone(), to.clone()));

                        Ok(())
                    }
                }
            }
        }
    }
}

pub mod fetch_many {
    use std::sync::Arc;

    use serde::Deserialize;
    use sqlx::{ColumnIndex, Decode, Encode, Type};
    use tokio::sync::RwLockReadGuard;

    use crate::{
        json_client::{
            database_for_json_client::DatabaseForJsonClient,
            fetch_many::extending_link_trait::JsonLinkFetchMany,
            json_client::JsonClient,
            links_utils::get_optional_to_many,
            supported_filter::{InvalidFilter, SupportedFilter, parse_supported_filter},
        },
        operations::{
            LinkedOutput, Operation,
            fetch_many::{FetchMany, ManyOutput},
        },
    };

    #[derive(Debug, Deserialize)]
    #[serde(tag = "ty")]
    pub enum SupportedLinkFetchMany {
        #[serde(rename = "optional_to_many")]
        OptionalToMany { to: String },
        #[serde(rename = "timestamp")]
        Timestamp,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct FetchManyInput {
        pub base: String,
        pub filters: Vec<SupportedFilter>,
        pub links: Vec<SupportedLinkFetchMany>,
        pub pagination: Pagination,
    }

    #[derive(Debug, Deserialize)]
    pub struct Pagination {
        pub limit: i64,
        pub first_item: Option<(i64, serde_json::Value)>,
        pub order_by: Option<String>,
    }

    pub type FetchManyOutput =
        ManyOutput<LinkedOutput<i64, serde_json::Value, Vec<serde_json::Value>>, (i64, ())>;

    #[derive(Debug)]
    pub enum FetchManyError {
        NoCollectionWithName(String),
        InvalidFilter(InvalidFilter),
    }

    impl<S> JsonClient<S>
    where
        S: DatabaseForJsonClient,
        i64: for<'q> Decode<'q, S> + for<'q> Encode<'q, S> + Type<S>,
        for<'s> &'s str: ColumnIndex<S::Row>,
    {
        pub fn fetch_many(
            &self,
            input: FetchManyInput,
        ) -> impl Future<Output = Result<FetchManyOutput, FetchManyError>> + Send {
            async move {
                let gaurd: RwLockReadGuard<'_, _> = self
                    .collections
                    .get(&input.base)
                    .ok_or(FetchManyError::NoCollectionWithName(input.base))?
                    .read()
                    .await;

                let base = gaurd.clone();

                let wheres = parse_supported_filter(input.filters, &base)
                    .map_err(FetchManyError::InvalidFilter)?;

                let links = {
                    let mut links = Vec::<Box<dyn JsonLinkFetchMany<S> + Send>>::new();
                    for each in input.links {
                        match each {
                            SupportedLinkFetchMany::OptionalToMany { to } => links.push(Box::new(
                                get_optional_to_many(&base, to, self).await.unwrap(),
                            )),
                            SupportedLinkFetchMany::Timestamp => {
                                todo!()
                            }
                        }
                    }
                    links
                };

                let mut conn = self.pool.begin().await.unwrap();

                let _ = ();
                let s = FetchMany {
                    base,
                    wheres,
                    links,
                    cursor_order_by: (),
                    cursor_first_item: None::<(i64, ())>,
                    limit: 10,
                };

                let out = Operation::<S>::exec_operation(s, &mut conn).await;

                drop(gaurd);

                Ok(out)
            }
        }
    }

    pub mod extending_link_trait {
        use serde::Serialize;
        use sqlx::Database;
        use tracing::warn;

        use crate::from_row::{FromRowAlias, FromRowData};
        use crate::operations::OperationOutput;
        use crate::operations::boxed_operation::BoxedOperation;
        use crate::operations::fetch_many::LinkFetch;
        use crate::query_builder::ManyBoxedExpressions;
        use crate::select_items_trait_object::SelectItemsTraitObject;
        use crate::select_items_trait_object::ToImplSelectItems;
        use crate::{database_extention::DatabaseExt, extentions::common_expressions::Aliased};
        use std::any::Any;
        use std::ops::{Deref, DerefMut};

        pub trait JsonLinkFetchMany<S> {
            fn select_items_expr(&self) -> Box<dyn SelectItemsTraitObject<S, ()>>;
            fn post_select_each_2(&self, item: &Box<dyn Any + Send>, poi: &mut Box<dyn Any + Send>);
            fn post_operation_input_init_2(&self) -> Box<dyn Any + Send>;
            fn post_select_2(
                &self,
                input: Box<dyn Any + Send>,
            ) -> Box<dyn BoxedOperation<S> + Send>;
            fn take_2(
                &self,
                item: Box<dyn Any + Send>,
                op: &mut Box<dyn Any + Send>,
            ) -> serde_json::Value;
            fn join_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
            fn wheres_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        }

        impl<S, T> JsonLinkFetchMany<S> for T
        where
            T: Clone + Send + 'static,
            T::SelectItems: Send,
            T::SelectItems: FromRowData,
            S: DatabaseExt,
            T: LinkFetch,
            T::SelectItems: Send
                + Aliased<
                    NumAliased: 'static + Send + ManyBoxedExpressions<S>,
                    Aliased: 'static + Send + ManyBoxedExpressions<S>,
                >,
            T::OpInput: 'static + Send,
            T::Op: Send + 'static + BoxedOperation<S>,
            T::Op: OperationOutput,
            T::Output: Serialize,
            T::SelectItems: FromRowData<RData: Send + 'static>,
            T::SelectItems: for<'r> FromRowAlias<'r, S::Row>,
            T::Join: Send + 'static + ManyBoxedExpressions<S>,
            T::Wheres: Send + 'static + ManyBoxedExpressions<S>,
        {
            fn join_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
                Box::new(self.non_duplicating_join_expressions())
            }
            fn wheres_expr(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
                Box::new(self.where_expressions())
            }
            fn take_2(
                &self,
                item: Box<dyn Any + Send>,
                op: &mut Box<dyn Any + Send>,
            ) -> serde_json::Value {
                let s = self.take_many(
                    *item
                        .downcast::<<T::SelectItems as FromRowData>::RData>()
                        .unwrap(),
                    op.downcast_mut::<<T::Op as OperationOutput>::Output>()
                        .unwrap(),
                );

                serde_json::to_value(s).expect("bug: serializing should not fail")
            }
            fn select_items_expr(&self) -> Box<dyn SelectItemsTraitObject<S, ()>> {
                Box::new(ToImplSelectItems {
                    select_items: self.non_aggregating_select_items(),
                    cast_from_row_result: (),
                })
            }
            fn post_select_each_2(
                &self,
                item: &Box<dyn Any + Send>,
                mut poi: &mut Box<dyn Any + Send>,
            ) {
                let ite_down = item
                    .deref()
                    .downcast_ref::<<T::SelectItems as FromRowData>::RData>();

                let poi_down = poi.deref_mut().downcast_mut::<T::OpInput>();

                self.operation_fix_on_many(ite_down.unwrap(), poi_down.unwrap())
            }

            fn post_operation_input_init_2(&self) -> Box<dyn Any + Send> {
                let ret = self.operation_initialize_input();

                Box::new(ret)
            }
            fn post_select_2(
                &self,
                input: Box<dyn Any + Send>,
            ) -> Box<dyn BoxedOperation<S> + Send> {
                Box::new(self.operation_construct(*input.downcast::<T::OpInput>().unwrap()))
            }
        }

        impl<'r, S> LinkFetch for Box<dyn JsonLinkFetchMany<S> + Send + 'r>
        where
            Box<dyn SelectItemsTraitObject<S, ()>>: FromRowData<RData = Box<dyn Any + Send>>,
            Box<dyn BoxedOperation<S> + Send>: OperationOutput<Output = Box<dyn Any + Send>>,
        {
            type SelectItems = Box<dyn SelectItemsTraitObject<S, ()>>;

            fn non_aggregating_select_items(&self) -> Self::SelectItems {
                self.select_items_expr()
            }

            fn operation_fix_on_many(&self, item: &Box<dyn Any + Send>, poi: &mut Self::OpInput)
            where
                Self::SelectItems: FromRowData,
            {
                self.post_select_each_2(item, poi)
            }

            fn take_many(
                &self,
                item: <Self::SelectItems as FromRowData>::RData,
                op: &mut <Self::Op as OperationOutput>::Output,
            ) -> Self::Output
            where
                Self::SelectItems: FromRowData,
                Self::Op: OperationOutput,
            {
                self.take_2(item, op)
            }

            type Join = Box<dyn ManyBoxedExpressions<S> + Send>;

            fn non_duplicating_join_expressions(&self) -> Self::Join {
                self.join_expr()
            }

            type Wheres = Box<dyn ManyBoxedExpressions<S> + Send>;

            fn where_expressions(&self) -> Self::Wheres {
                self.wheres_expr()
            }

            type Output = serde_json::Value;

            type OpInput = Box<dyn Any + Send>;

            fn operation_initialize_input(&self) -> Self::OpInput {
                let ret = self.post_operation_input_init_2();

                ret
            }

            type Op = Box<dyn BoxedOperation<S> + Send>;

            fn operation_construct(&self, input: Self::OpInput) -> Self::Op
            where
                Self::SelectItems: FromRowData,
            {
                self.post_select_2(input)
            }
        }

        impl<'r, S> LinkFetch for Vec<Box<dyn JsonLinkFetchMany<S> + Send + 'r>>
        where
            S: Database,
            Vec<Box<dyn SelectItemsTraitObject<S, ()>>>:
                FromRowData<RData = Vec<Box<dyn Any + Send>>>,
            Vec<Box<dyn BoxedOperation<S> + Send>>:
                OperationOutput<Output = Vec<Box<dyn Any + Send>>>,
        {
            type SelectItems = Vec<Box<dyn SelectItemsTraitObject<S, ()>>>;

            fn non_aggregating_select_items(&self) -> Self::SelectItems {
                let s = self
                    .iter()
                    .map(|each| each.select_items_expr())
                    .collect::<Vec<_>>();

                s
            }

            fn operation_fix_on_many(
                &self,
                item: &Vec<Box<dyn Any + Send>>,
                poi: &mut Vec<Box<dyn Any + Send>>,
            ) where
                Self::SelectItems: FromRowData,
            {
                for (i, each) in self.iter().enumerate() {
                    let corresponding_poi = poi.get_mut(i).unwrap();
                    let corresponding_item = item.get(i).unwrap();
                    each.post_select_each_2(corresponding_item, corresponding_poi);
                }
            }

            fn take_many(
                &self,
                item: Vec<Box<dyn Any + Send>>,
                op: &mut Vec<Box<dyn Any + Send>>,
            ) -> Self::Output
            where
                Self::SelectItems: FromRowData,
                Self::Op: OperationOutput,
            {
                let mut ret = vec![];
                for (i, (each, item)) in self.iter().zip(item.into_iter()).enumerate() {
                    let corresponding_op = op.get_mut(i).unwrap();
                    ret.push(each.take_many(item, corresponding_op));
                }

                ret
            }

            type Join = Box<dyn ManyBoxedExpressions<S> + Send>;

            fn non_duplicating_join_expressions(&self) -> Self::Join {
                let first = self.iter().next().unwrap();
                warn!("multiple links");
                Box::new(first.join_expr())
            }

            type Wheres = ();

            fn where_expressions(&self) -> Self::Wheres {}

            type Output = Vec<serde_json::Value>;

            type OpInput = Vec<Box<dyn Any + Send>>;

            fn operation_initialize_input(&self) -> Self::OpInput {
                let ret = self
                    .iter()
                    .map(|each| each.post_operation_input_init_2())
                    .collect::<Vec<_>>();
                ret
            }

            type Op = Vec<Box<dyn BoxedOperation<S> + Send>>;

            fn operation_construct(&self, input: Self::OpInput) -> Self::Op
            where
                Self::SelectItems: FromRowData,
            {
                let mut ret = vec![];
                for (each, input) in self.iter().zip(input.into_iter()) {
                    let res = each.operation_construct(input);
                    ret.push(res);
                }
                ret
            }
        }
    }
}

#[claw_ql_macros::skip]
pub mod fetch_one {
    use super::database_for_json_client::DatabaseForJsonClient;
    use super::json_client::JsonClient;
    use crate::json_client::supported_filter::InvalidFilter;
    use crate::json_client::supported_filter::SupportedFilter;
    use crate::operations::LinkedOutput;
    use crate::operations::Operation;
    use crate::operations::fetch_one::FetchOne;
    use serde::{Deserialize, Serialize};
    use serde_json::Value as JsonValue;
    use sqlx::{ColumnIndex, Decode, Executor, Type};
    use tokio::sync::RwLockReadGuard;

    #[derive(Debug, Deserialize)]
    #[serde(tag = "ty")]
    pub enum SupportedLinkFetchOne {
        #[serde(rename = "optional_to_many")]
        OptionalToMany { to: String },
    }

    #[derive(Debug, Deserialize)]
    pub struct FetchOneInput {
        pub base: String,
        pub links: Vec<SupportedLinkFetchOne>,
        pub filters: Vec<SupportedFilter>,
    }

    pub type FetchOneOutput = Option<LinkedOutput<i64, JsonValue, Vec<JsonValue>>>;

    pub enum FetchOneError {
        NoCollectionWithName(String),
        InvalidFilter(InvalidFilter),
    }

    impl<S> JsonClient<S>
    where
        S: DatabaseForJsonClient,
        for<'q> &'q str: ColumnIndex<S::Row>,
        for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
        for<'q> i64: Decode<'q, S> + Type<S>,
    {
        pub fn fetch_one(
            &self,
            input: FetchOneInput,
        ) -> impl Future<Output = Result<FetchOneOutput, FetchOneError>> + Send {
            async move {
                let gaurd: RwLockReadGuard<'_, _> = self
                    .collections
                    .get(&input.base)
                    .ok_or(FetchOneError::NoCollectionWithName(input.base))?
                    .read()
                    .await;

                let base = gaurd.clone();

                let mut tx = self.pool.begin().await.unwrap();

                let ret = Operation::<S>::exec_operation(
                    FetchOne {
                        wheres: (),
                        links: {
                            use super::json_link_fetch_one_extention::JsonLinkFetchOne;
                            use super::supported_links_on_fetch_one::{
                                match_and_cast_to_fetch, on_request,
                            };
                            let mut s = Vec::<Box<dyn JsonLinkFetchOne<S> + 'static + Send>>::new();
                            // let mut s = vec![];
                            for (index, link) in input.links.into_iter().enumerate() {
                                let index = index as u64;
                                let v = on_request(link, base.name_lower_case.clone(), self)
                                    .map_err(|e| (index, e))?;
                                let v = match_and_cast_to_fetch(v, self).await;
                                s.push(v);
                            }
                            // vec![Box::new(()) as Box<dyn JsonLinkFetchOne<S> + Send>]
                            s
                        },
                        base,
                    },
                    tx.as_mut(),
                )
                .await;

                tx.commit().await.unwrap();

                // hold guard to delay modification until this operation is finished
                drop(gaurd);

                Ok(ret)
            }
        }
    }
}

#[claw_ql_macros::skip]
// to be refactored
pub mod json_link_fetch_one_extention {
    use std::any::Any;

    use crate::{
        database_extention::DatabaseExt,
        from_row::RowTwoAliased,
        operations::{
            BoxedOperation, Operation,
            fetch_one::{LinkFetchOne, link_select_item},
        },
        query_builder::{ManyExpressions, functional_expr::ManyBoxedExpressions},
    };
    use serde::Serialize;
    use serde_json::Value as JsonValue;
    use serde_json::to_value;
    use sqlx::Database;

    pub trait JsonLinkFetchOne<S>: 'static + Send
    where
        S: Database,
    {
        fn non_aggregating_select_items(&self) -> Vec<link_select_item<String, String>>;
        fn non_duplicating_joins(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn wheres(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        fn sub_op(
            &self,
            row: two_alias<'_, <S as Database>::Row>,
        ) -> (Box<dyn BoxedOperation<S> + Send>, Box<dyn Any + Send>);
        fn take(
            self: Box<Self>,
            sub_op: Box<dyn Any + Send>,
            inner: Box<dyn Any + Send>,
        ) -> JsonValue;
    }

    impl<S, T> JsonLinkFetchOne<S> for T
    where
        S: DatabaseExt,
        T: Send + 'static,
        T: LinkFetchOne<
                S,
                SubOp: Send + Operation<S, Output: Send>,
                Inner: 'static + Send,
                Output: Serialize,
            >,
        T::Joins: Send + ManyExpressions<'static, S>,
        T::Wheres: Send + ManyExpressions<'static, S>,
        S: 'static + Database,
    {
        fn non_aggregating_select_items(&self) -> Vec<link_select_item<String, String>> {
            <T as LinkFetchOne<S>>::non_aggregating_select_items(self)
        }
        fn non_duplicating_joins(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(T::non_duplicating_joins(self))
        }
        fn wheres(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
            Box::new(T::wheres(self))
        }
        fn sub_op(
            &self,
            row: two_alias<'_, <S as sqlx::Database>::Row>,
        ) -> (
            Box<dyn BoxedOperation<S> + std::marker::Send + 'static>,
            Box<dyn Any + Send + 'static>,
        ) {
            let su = <T as LinkFetchOne<S>>::sub_op(self, row);
            (Box::new(su.0), Box::new(su.1))
        }
        fn take(
            self: Box<Self>,
            sub_op: Box<dyn Any + Send>,
            inner: Box<dyn Any + Send>,
        ) -> JsonValue {
            let inner = inner
                .downcast::<<T as LinkFetchOne<S>>::Inner>()
                .expect("bug: blanket impl should be consistant");

            let sub_op = sub_op
                .downcast::<<<T as LinkFetchOne<S>>::SubOp as Operation<S>>::Output>()
                .expect("bug: blanket impl should be consistant");

            to_value(<T as LinkFetchOne<S>>::take(*self, *sub_op, *inner))
                .expect("bug: serializing should not fail")
        }
    }

    impl<'q, S: Database + 'static> LinkFetchOne<S> for Box<dyn JsonLinkFetchOne<S> + Send + 'q> {
        type Joins = Vec<Box<dyn ManyBoxedExpressions<S> + Send>>;
        type Wheres = Vec<Box<dyn ManyBoxedExpressions<S> + Send>>;

        fn non_aggregating_select_items(&self) -> Vec<link_select_item<String, String>> {
            JsonLinkFetchOne::non_aggregating_select_items(&**self)
        }
        fn non_duplicating_joins(&self) -> Self::Joins {
            vec![JsonLinkFetchOne::non_duplicating_joins(&**self)]
        }
        fn wheres(&self) -> Self::Wheres {
            vec![JsonLinkFetchOne::wheres(&**self)]
        }

        type Inner = Box<dyn Any + Send>;

        type SubOp = Box<dyn BoxedOperation<S> + Send>;

        fn sub_op(
            &self,
            row: two_alias<'_, <S as sqlx::Database>::Row>,
        ) -> (Self::SubOp, Self::Inner)
        where
            S: sqlx::Database,
        {
            JsonLinkFetchOne::sub_op(&**self, row)
        }

        type Output = JsonValue;

        fn take(
            self,
            extend: <Self::SubOp as Operation<S>>::Output,
            inner: Self::Inner,
        ) -> Self::Output {
            JsonLinkFetchOne::take(self, extend, inner)
        }
    }
}

#[claw_ql_macros::skip]
// to be refactored
pub mod update_one {
    use crate::json_client::database_for_json_client::DatabaseForJsonClient;
    use crate::json_client::json_client::JsonClient;
    use crate::json_client::supported_filters::{InvalidFilter, SupportedFilter};
    use crate::operations::{self, SafeOperation};
    use crate::operations::{LinkedOutput, Operation, update_one::UpdateOne};
    use serde::Deserialize;
    use serde_json::Error as DeserializeError;
    use serde_json::Value as JsonValue;
    use tokio::sync::RwLockReadGuard;

    #[derive(Deserialize)]
    pub struct UpdateOneInput {
        pub base: String,
        pub partial: JsonValue,
        pub links: Vec<()>,
        pub filters: Vec<SupportedFilter<JsonValue>>,
    }

    pub type UpdateOneOutput = Option<LinkedOutput<i64, JsonValue, ()>>;

    #[derive(Debug)]
    pub enum UpdateOneError {
        NoCollectionWithName(String),
        UpdateDataInvalid(DeserializeError),
        DeserializeError(DeserializeError),
        InvalidFilter(InvalidFilter),
        NonOperational,
        NonUniqueFilters,
    }

    impl From<operations::update_one::UpdateOneError<serde_json::Error>> for UpdateOneError {
        fn from(value: operations::update_one::UpdateOneError<serde_json::Error>) -> Self {
            match value {
                operations::update_one::UpdateOneError::ValidationError(d) => {
                    Self::UpdateDataInvalid(d)
                }
                operations::update_one::UpdateOneError::NonUniqueOperation => {
                    Self::NonUniqueFilters
                }
                operations::update_one::UpdateOneError::NonOperational => Self::NonOperational,
            }
        }
    }

    impl From<InvalidFilter> for UpdateOneError {
        fn from(value: InvalidFilter) -> Self {
            Self::InvalidFilter(value)
        }
    }

    impl<S> JsonClient<S>
    where
        S: DatabaseForJsonClient,
        usize: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        for<'s> &'s str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        i64: sqlx::Type<S> + for<'q> sqlx::Decode<'q, S>,
        for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
    {
        pub fn update_one(
            &self,
            input: UpdateOneInput,
        ) -> impl Future<Output = Result<UpdateOneOutput, UpdateOneError>> + Send {
            async move {
                let gaurd: RwLockReadGuard<'_, _> = self
                    .collections
                    .get(&input.base)
                    .ok_or(UpdateOneError::NoCollectionWithName(input.base))?
                    .read()
                    .await;

                let handler = gaurd.clone();

                let mut tx = self.pool.begin().await.unwrap();

                let ret = Operation::<S>::exec_operation(
                    UpdateOne {
                        wheres: {
                            let mut ret = vec![];
                            for filter in input.filters {
                                ret.push(filter.safety_check(&handler)?);
                            }
                            ret
                        },
                        handler: handler,
                        partial: input.partial,
                        links: (),
                    }
                    .safety_check()?,
                    tx.as_mut(),
                )
                .await;

                tx.commit().await.unwrap();

                // hold guard to delay modification until this operation is finished
                drop(gaurd);

                Ok(ret)
            }
        }
    }
}

#[claw_ql_macros::skip]
// to be refactored
pub mod supported_links_on_fetch_one {
    use std::collections::HashSet;
    use std::ops::Not;

    use crate::json_client::OnlyDefaultLinks;
    use crate::json_client::json_client::JsonClient;
    use crate::json_client::json_link_fetch_one_extention::JsonLinkFetchOne;
    use crate::{
        json_client::{
            database_for_json_client::DatabaseForJsonClient, dynamic_collection::DynamicCollection,
        },
        links::{Link, relation_optional_to_many::OptionalToMany},
    };
    use serde::Deserialize;
    use serde_json::Value as JsonValue;
    use sqlx::{Decode, prelude::Type};

    #[derive(Default)]
    pub struct LinkInformations {
        pub optional_to_many: HashSet<(String, String)>,
    }

    #[derive(Deserialize)]
    pub struct OnFetchRequest {
        pub ty: String,
        pub to: Option<String>,
    }

    #[derive(Debug, serde::Serialize)]
    pub enum InvalidLink {
        CollectionDoesntExist(String),
        CollectionsAreNotLinked(String, String),
        InvalidInput,
    }

    pub fn on_request<S: DatabaseForJsonClient>(
        value: JsonValue,
        base: String,
        jc: &JsonClient<S>,
    ) -> Result<SupportedLink<String>, InvalidLink> {
        let j: OnFetchRequest =
            serde_json::from_value(value).map_err(|e| InvalidLink::InvalidInput)?;

        match (j.ty.as_str(), j.to) {
            ("optional_to_many", Some(to)) => {
                if jc.collections.keys().any(|k| k == &to).not() {
                    return Err(InvalidLink::CollectionDoesntExist(to));
                }
                if jc.collections.keys().any(|k| k == &base).not() {
                    return Err(InvalidLink::CollectionDoesntExist(base));
                }
                if jc
                    .links
                    .optional_to_many
                    .contains(&(base.clone(), to.clone()))
                    .not()
                {
                    return Err(InvalidLink::CollectionsAreNotLinked(base, to));
                }

                Ok(SupportedLink::OptionalToMany(OptionalToMany {
                    foriegn_key: OnlyDefaultLinks,
                    from: base,
                    to,
                }))
            }
            _ => Err(InvalidLink::InvalidInput),
        }
    }

    pub fn match_and_cast_to_fetch<S>(
        this: SupportedLink<String>,
        jc: &JsonClient<S>,
    ) -> impl Future<Output = Box<dyn JsonLinkFetchOne<S> + 'static + Send>>
    where
        S: DatabaseForJsonClient,
        i64: for<'q> Decode<'q, S> + Type<S>,
        for<'q> &'q str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
    {
        async move {
            match this {
            SupportedLink::OptionalToMany(v) => Box::new(OptionalToMany {
                foriegn_key: OnlyDefaultLinks,
                from: jc
                    .collections
                    .get(&v.from)
                    .expect("bug: as long as no breaking-change occured between `on_request` and `match_and_cast_to_fetch`, this should never happen")
                    .read()
                    .await
                    .clone(),
                to: jc
                    .collections
                    .get(&v.to)
                    .expect("bug: as long as no breaking-change occured between `on_request` and `match_and_cast_to_fetch`, this should never happen")
                    .read()
                    .await
                    .clone(),
            }) as Box<dyn JsonLinkFetchOne<S> + Send>,
        }
        }
    }

    pub enum SupportedLink<DC> {
        OptionalToMany(OptionalToMany<OnlyDefaultLinks, DC, DC>),
    }

    impl<'a, S> Link<DynamicCollection<S>> for Box<dyn JsonLinkFetchOne<S> + Send + 'a> {
        type Spec = Self;
        fn spec(self, _: &DynamicCollection<S>) -> Self::Spec {
            self
        }
    }
}

#[claw_ql_macros::skip]
// to be refactored
pub mod insert_one {
    use super::database_for_json_client::DatabaseForJsonClient;
    use super::json_client::JsonClient;

    use crate::{
        operations::{self, LinkedOutput, Operation, SafeOperation},
        prelude::sql::InsertOne,
    };
    use serde_json::Error as DeserializeError;
    use serde_json::Value as JsonValue;
    use tokio::sync::RwLockReadGuard;

    #[derive(Debug)]
    pub struct InsertOneInput {
        pub base: String,
        pub data: JsonValue,
        pub links: Vec<JsonValue>,
    }
    pub type InsertOneOutput = LinkedOutput<i64, (), ()>;

    #[derive(Debug)]
    // #[simple_enum]
    pub enum InsertOneError {
        NoCollectionWithName(String),
        DataInvalid(serde_json::Error),
        DeserializeError(DeserializeError),
    }

    impl From<operations::insert_one::InsertOneError<serde_json::Error>> for InsertOneError {
        fn from(value: operations::insert_one::InsertOneError<serde_json::Error>) -> Self {
            match value {
                operations::insert_one::InsertOneError::ValidationError(s) => {
                    return Self::DataInvalid(s);
                }
            }
        }
    }

    impl<S> JsonClient<S>
    where
        S: DatabaseForJsonClient,
        usize: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        i64: sqlx::Type<S> + for<'q> sqlx::Decode<'q, S>,
        for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
    {
        pub fn insert_one(
            &self,
            input: InsertOneInput,
        ) -> impl Future<Output = Result<InsertOneOutput, InsertOneError>> + Send {
            async move {
                let gaurd: RwLockReadGuard<'_, _> = self
                    .collections
                    .get(&input.base)
                    .ok_or(InsertOneError::NoCollectionWithName(input.base))?
                    .read()
                    .await;

                let handler = gaurd.clone();

                let mut tx = self.pool.begin().await.unwrap();

                let ret = Operation::<S>::exec_operation(
                    InsertOne {
                        handler,
                        data: input.data,
                        links: (),
                    }
                    .safety_check()?,
                    tx.as_mut(),
                )
                .await;

                tx.commit().await.unwrap();

                // hold guard to delay modification until this operation is finished
                drop(gaurd);

                Ok(ret)
            }
        }
    }
}

#[claw_ql_macros::skip]
// to be refactored
pub mod delete_one {
    use crate::json_client::database_for_json_client::DatabaseForJsonClient;
    use crate::json_client::json_client::JsonClient;
    use crate::json_client::supported_filters::{InvalidFilter, SupportedFilter};
    use crate::operations::delete_one::DeleteOne;
    use crate::operations::{self, SafeOperation};
    use crate::operations::{LinkedOutput, Operation};
    use serde::Deserialize;
    use serde_json::Value as JsonValue;
    use tokio::sync::RwLockReadGuard;

    #[derive(Deserialize)]
    pub struct DeleteOneInput {
        pub base: String,
        pub filters: Vec<SupportedFilter<JsonValue>>,
        pub links: Vec<()>,
    }

    pub type DeleteOneOutput = Option<LinkedOutput<i64, JsonValue, ()>>;

    #[derive(Debug)]
    pub enum DeleteOneError {
        NoCollectionWithName(String),
        // UpdateDataInvalid(DeserializeError),
        // DeserializeError(DeserializeError),
        // NonOperational,
        NonUniqueFilters,
        InvalidWhere(InvalidFilter),
    }

    impl From<operations::delete_one::DeleteOneError> for DeleteOneError {
        fn from(value: operations::delete_one::DeleteOneError) -> Self {
            match value {
                operations::delete_one::DeleteOneError::NonUniqueFilters => Self::NonUniqueFilters,
            }
        }
    }

    impl From<InvalidFilter> for DeleteOneError {
        fn from(value: InvalidFilter) -> Self {
            Self::InvalidWhere(value)
        }
    }

    impl<S> JsonClient<S>
    where
        S: DatabaseForJsonClient,
        usize: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        for<'q> &'q str: sqlx::ColumnIndex<<S as sqlx::Database>::Row>,
        i64: sqlx::Type<S> + for<'q> sqlx::Encode<'q, S> + for<'q> sqlx::Decode<'q, S>,
        for<'c> &'c mut <S as sqlx::Database>::Connection: sqlx::Executor<'c, Database = S>,
    {
        pub fn delete_one(
            &self,
            input: DeleteOneInput,
        ) -> impl Future<Output = Result<DeleteOneOutput, DeleteOneError>> + Send {
            async move {
                let gaurd: RwLockReadGuard<'_, _> = self
                    .collections
                    .get(&input.base)
                    .ok_or(DeleteOneError::NoCollectionWithName(input.base))?
                    .read()
                    .await;

                let handler = gaurd.clone();

                let mut tx = self.pool.begin().await.unwrap();

                let ret = Operation::<S>::exec_operation(
                    DeleteOne {
                        wheres: {
                            let mut ret = vec![];
                            for filter in input.filters {
                                ret.push(filter.safety_check(&handler)?);
                            }
                            ret
                        },
                        handler,
                        links: (),
                    }
                    .safety_check()?,
                    tx.as_mut(),
                )
                .await;

                tx.commit().await.unwrap();

                // hold guard to delay modification until this operation is finished
                drop(gaurd);

                Ok(ret)
            }
        }
    }
}

#[claw_ql_macros::skip]
mod as_router {
    use super::database_for_json_client::DatabaseForJsonClient;
    use super::json_client::JsonClient;
    use crate::json_client::fetch_one::FetchOneInput;
    use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
    use sqlx::{ColumnIndex, Decode, Encode, prelude::Type};
    use std::sync::Arc;
    use tokio::sync::RwLock as TrwLock;

    impl<S> JsonClient<S>
    where
        S: DatabaseForJsonClient,
        for<'c> &'c mut S::Connection: sqlx::Executor<'c, Database = S>,
        for<'e> i64: Encode<'e, S> + Type<S> + Decode<'e, S>,
        for<'e> &'e str: ColumnIndex<S::Row>,
    {
        pub fn as_router(self) -> Router<()> {
            Router::new()
            .route(
                "/fetch_one",
                get(
                    |jc: State<Arc<TrwLock<JsonClient<S>>>>, body: Json<FetchOneInput>| async move {
                        let jc = jc.read().await;
                        let res = jc.fetch_one(body.0).await;
                        Json(res).into_response()
                    },
                ),
            )
            .with_state(Arc::new(TrwLock::new(self)))
        }
    }
}

// maybe usefull for ExtendableJsonClient in the future
#[claw_ql_macros::skip]
mod old_code {
    mod old_fetch_one {

        /// reflexive impl -- errors has been cleared on JsonLink::on_request* and JsonLink::create_link
        impl<S> Link<Arc<dyn JsonCollection<S>>> for Box<dyn JsonLinkFetchOne<S>> {
            type Spec = Self;

            fn spec(self, _: &Arc<dyn JsonCollection<S>>) -> Self::Spec {
                self
            }
        }

        /// JsonLink extention
        pub(super) fn on_fetch_one_request<S, T>(
            jc: &T,
            base: Arc<dyn JsonCollection<S> + Send + Sync>,
            input: JsonValue,
        ) -> Result<Box<dyn JsonLinkFetchOne<S>>, JsonValue>
        where
            S: Database,
            T: DynamicLink<Arc<dyn JsonCollection<S> + Send + Sync>>,
            T::OnRequestInput: DeserializeOwned,
            T::OnRequestError: Serialize,
            T::OnRequest: JsonLinkFetchOne<S>,
        {
            let req = jc
                .on_request(&base, from_value(input).unwrap())
                .map_err(|e| to_value(e).unwrap())
                .unwrap();
            Ok(Box::new(req))
        }
    }

    mod router_testing {
        use crate::connect_in_memory::ConnectInMemory;
        use crate::json_client::JsonClient;
        use axum::Router;
        use axum::body::Body;
        use axum::extract::Request;
        use axum::response::Response;
        use futures::TryStreamExt;
        use hyper::Method;
        use serde_json::Value;
        use serde_json::json;
        use sqlx::Sqlite;
        use tower::ServiceExt;

        async fn test_router_as_json(
            router: Router<()>,
            uri: &str,
            body: Value,
        ) -> Result<Value, String> {
            let res: Response<Body> = router
                .oneshot(
                    Request::builder()
                        .uri(uri)
                        .header(hyper::header::CONTENT_TYPE, "application/json")
                        .method(Method::GET)
                        .body(body.to_string())
                        .map_err(|e| e.to_string())?,
                )
                .await
                .map_err(|e| e.to_string())?;

            // axum::Body to bytes
            let res = res
                .into_body()
                .into_data_stream()
                .try_collect::<Vec<_>>()
                .await
                .map_err(|e| e.to_string())?;

            // from bytes to bits
            let res = res.into_iter().flatten().collect::<Vec<_>>();
            // bits to string
            let res = String::from_utf8(res).map_err(|e| e.to_string())?;

            Ok(serde_json::from_str(&res).map_err(|e| e.to_string())?)
        }

        #[tokio::test]
        async fn client() {
            let pool = Sqlite::connect_in_memory().await;

            let jc = JsonClient::from(pool);

            let router = jc.as_router();

            let res = test_router_as_json(
                router.clone(),
                "/fetch_one",
                json!({
                    "base": "todo",
                    "wheres": [],
                    "link": [],
                }),
            )
            .await
            .unwrap();

            pretty_assertions::assert_eq!(
                res,
                json!({
                    "Err": {
                        "NoCollectionWithName": "todo",
                    }
                })
            );
        }
    }

    #[claw_ql_macros::skip]
    pub mod json_collection_trait {
        use std::ops::Not;

        use serde::{Serialize, de::DeserializeOwned};
        use serde_json::Value as JsonValue;
        use serde_json::to_value;
        use sqlx::Database;

        use crate::{
            collections::{Collection, CollectionBasic, SingleIncremintalInt},
            database_extention::DatabaseExt,
            extentions::Members,
            from_row::FromRowAlias,
            query_builder::{
                IsOpExpression, ManyExpressions, functional_expr::ManyBoxedExpressions,
            },
        };

        pub type DeserializeError = serde_json::Error;

        pub trait JsonCollection<S>: Send + Sync {
            fn data_to_expression(
                &self,
                data: JsonValue,
            ) -> Result<Box<dyn ManyBoxedExpressions<S> + Send>, DeserializeError>;
            fn partial_to_expressions(
                &self,
                partial: JsonValue,
            ) -> Result<Box<dyn ManyBoxedExpressions<S> + Send>, DeserializeError>;
            fn table_name(&self) -> &str;
            fn table_name_lower_case(&self) -> &str;
            fn from_row_two_alias<'r>(
                &self,
                row: crate::from_row::RowTwoAliased<'r, S::Row>,
            ) -> JsonValue
            where
                S: Database;
            fn from_row_pre_alias<'r>(
                &self,
                row: crate::from_row::RowPreAliased<'r, S::Row>,
            ) -> JsonValue
            where
                S: Database;
            fn members(&self) -> Vec<String>;
        }

        impl<T, S> JsonCollection<S> for T
        where
            T: Send + Sync + 'static,
            S: DatabaseExt,
            <T as Collection>::Data: Send + Serialize + DeserializeOwned,
            <T as Collection>::Data: ManyExpressions<'static, S>,
            <T as Collection>::Partial: Send + Serialize + DeserializeOwned,
            <T as Collection>::Partial: ManyExpressions<'static, S>,
            T: Collection<Id = SingleIncremintalInt>,
            T: for<'r> FromRowAlias<'r, S::Row, FromRowData = T::Data>,
            T: Members,
        {
            fn data_to_expression(
                &self,
                data: JsonValue,
            ) -> Result<Box<dyn ManyBoxedExpressions<S> + Send>, DeserializeError> {
                let data = serde_json::from_value::<<T as Collection>::Data>(data)?;
                Ok(Box::new(data))
            }
            fn partial_to_expressions(
                &self,
                partial: JsonValue,
            ) -> Result<Box<dyn ManyBoxedExpressions<S> + Send>, DeserializeError> {
                let partial = serde_json::from_value::<<T as Collection>::Partial>(partial)?;
                Ok(Box::new(partial))
            }
            fn table_name(&self) -> &str {
                CollectionBasic::table_name(self)
            }

            fn from_row_two_alias<'r>(
                &self,
                row: crate::from_row::RowTwoAliased<'r, <S>::Row>,
            ) -> JsonValue
            where
                S: Database,
            {
                to_value(T::two_alias(self, row).expect("sound claw_ql code"))
                    .expect("sound claw_ql code")
            }

            fn table_name_lower_case(&self) -> &str {
                CollectionBasic::table_name_lower_case(self)
            }
            fn from_row_pre_alias<'r>(
                &self,
                row: crate::from_row::RowPreAliased<'r, <S as sqlx::Database>::Row>,
            ) -> JsonValue
            where
                S: Database,
            {
                to_value(T::pre_alias(self, row).expect("sound claw_ql code"))
                    .expect("sound value impl")
            }
            fn members(&self) -> Vec<String> {
                Members::members_names(self)
            }
        }

        mod expression_impls {
            use super::*;
            use crate::database_extention::DatabaseExt;
            use crate::query_builder::{IsOpExpression, ManyExpressions};
            use std::sync::Arc;

            impl<'q, S: 'q> IsOpExpression for Arc<dyn JsonCollection<S> + Send + Sync + 'q> {
                fn is_op(&self) -> bool {
                    Members::members_names(self).is_empty().not()
                }
            }

            impl<'q, S: 'q> ManyExpressions<'q, S> for Arc<dyn JsonCollection<S> + Send + Sync + 'q> {
                fn expression<
                    Start: crate::query_builder::SqlSyntax + ?Sized,
                    Join: crate::query_builder::SqlSyntax + ?Sized,
                >(
                    self,
                    start: &Start,
                    join: &Join,
                    ctx: &mut crate::query_builder::StatementBuilder<'q, S>,
                ) where
                    S: crate::database_extention::DatabaseExt,
                {
                    let members = self.members_names();
                    if members.is_empty().not() {
                        ctx.syntax(start);
                    }
                    for (index, member) in members.iter().enumerate() {
                        ctx.sanitize(member.as_str());
                        if index < members.len() - 1 {
                            ctx.syntax(join);
                        }
                    }
                }
            }
        }

        mod from_row_impls {
            use super::*;
            use crate::database_extention::DatabaseExt;
            use crate::query_builder::{IsOpExpression, ManyExpressions};
            use std::sync::Arc;

            impl<'r, 'a, S: Database> FromRowAlias<'r, S::Row>
                for Arc<dyn JsonCollection<S> + Send + Sync + 'a>
            {
                type FromRowData = JsonValue;
                fn no_alias(
                    &self,
                    _: &'r S::Row,
                ) -> Result<Self::FromRowData, crate::from_row::FromRowError> {
                    todo!("impl no alias")
                }

                fn two_alias(
                    &self,
                    row: crate::from_row::RowTwoAliased<'r, S::Row>,
                ) -> Result<Self::FromRowData, crate::from_row::FromRowError>
                where
                    S::Row: sqlx::Row,
                {
                    Ok(JsonCollection::from_row_two_alias(&**self, row))
                }

                fn pre_alias(
                    &self,
                    row: crate::from_row::RowPreAliased<'r, S::Row>,
                ) -> Result<Self::FromRowData, crate::from_row::FromRowError> {
                    Ok(JsonCollection::from_row_pre_alias(&**self, row))
                }

                fn post_alias(
                    &self,
                    _: crate::from_row::RowPostAliased<'r, S::Row>,
                ) -> Result<Self::FromRowData, crate::from_row::FromRowError> {
                    todo!("impl post alias")
                }
            }
        }

        mod collection_impls {
            use super::*;
            use crate::collections::Collection;
            use crate::collections::ValidateCollection;
            use serde_json::Value as JsonValue;
            use std::sync::Arc;

            impl<'r, S: 'r> Collection for Arc<dyn JsonCollection<S> + Send + Sync + 'r> {
                fn table_name(&self) -> &str {
                    todo!()
                }
                fn table_name_lower_case(&self) -> &str {
                    todo!()
                }
                type Partial = JsonValue;
                type Data = JsonValue;
                // type PartialInput = JsonValue;
                // type PartialValidationError = DeserializeError;
                // type Partial = JsonValue;
                // fn validate_partial(
                //     &self,
                //     _: Self::PartialInput,
                // ) -> Result<Self::Partial, Self::PartialValidationError> {
                //     todo!()
                // }
                // type DataInput = JsonValue;
                // type DataValidationError = DeserializeError;
                // type Data = JsonValue;
                // fn validate_data(
                //     &self,
                //     _: Self::DataInput,
                // ) -> Result<Self::Data, Self::DataValidationError> {
                //     todo!()
                // }
                type Id = SingleIncremintalInt;
                fn id(&self) -> &Self::Id {
                    &SingleIncremintalInt
                }
            }

            impl<'r, S: 'r> ValidateCollection for Arc<dyn JsonCollection<S> + Send + Sync + 'r> {
                type PartialInput = JsonValue;
                type PartialValidationError = DeserializeError;
                type PartialOk = JsonValue;
                fn validate_partial(
                    &self,
                    _: Self::PartialInput,
                ) -> Result<Self::Partial, Self::PartialValidationError> {
                    todo!()
                }
                type DataInput = JsonValue;
                type DataValidationError = DeserializeError;
                type Data = JsonValue;
                fn validate_data(
                    &self,
                    _: Self::DataInput,
                ) -> Result<Self::Data, Self::DataValidationError> {
                    todo!()
                }
            }
        }

        // impl<'r, S: 'static> ValidateCollection for Dyn {
        //     type PartialInput = JsonValue;
        //     type PartialValidationError = DeserializeError;
        //     type Partial = JsonPartial<S>;
        //     fn validate_partial(
        //         &self,
        //         input: Self::PartialInput,
        //     ) -> Result<Self::Partial, Self::PartialValidationError> {
        //         let mut v: Vec<Box<dyn ToBind<S> + Send>> = vec![];

        //         let mut map = from_value::<HashMap<String, JsonValue>>(input)?;

        //         for field in self.fields.iter() {
        //             match (&field.is_optional, map.remove(&field.name)) {
        //                 (true, None) => v.push(Box::new(())),
        //                 (false, None) => {
        //                     return Err(de::Error::custom("cannot find filed with name "));
        //                 }
        //                 (_, Some(value)) => {
        //                     v.push(field.type_info.to_bind(value)?);
        //                 }
        //             }
        //         }

        //         Ok(JsonPartial { vec: v })
        //     }

        //     type Data = JsonValue;

        //     type Id = SingleIncremintalInt;
        //     fn id(&self) -> &Self::Id {
        //         &SingleIncremintalInt
        //     }
        // }

        impl<'r, S: 'r> Members for Arc<dyn JsonCollection<S> + Send + Sync + 'r> {
            fn members_names(&self) -> Vec<String> {
                JsonCollection::members(&**self)
            }
        }
    }

    mod add_collection {
        use claw_ql_macros::skip;
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
            ) -> impl FnOnce(&mut <S as QueryBuilder>::Context2) -> String + 'static + use<S>
            {
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
            ) -> impl FnOnce(&mut <S as QueryBuilder>::Context2) -> String + 'static + use<S>
            {
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

                let va =
                    serde_json::to_value(ret).expect("shoudn't serializing error be outruled?");

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
            pub async fn add_collection(
                &mut self,
                body: AddCollectionBody,
            ) -> Result<(), CollectionExist> {
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
    }

    // mod operations_for_test_purposes {
    //     use super::*;
    //     use crate::database_extention::DatabaseExt;
    //     use serde_json::{from_value, json, to_value};
    //     use sqlx::{ColumnIndex, Decode, Executor, Type};

    //     impl<S: Database> JsonClient<S> {
    //         pub fn json_operation(
    //             &self,
    //             operation: JsonValue,
    //         ) -> impl Future<Output = Result<JsonValue, JsonValue>> + Send
    //         where
    //             S: DatabaseExt,
    //             for<'q> &'q str: ColumnIndex<S::Row>,
    //             for<'c> &'c mut <S as sqlx::Database>::Connection: Executor<'c, Database = S>,
    //             for<'q> i64: Decode<'q, S> + Type<S>,
    //         {
    //             async move {
    //                 match operation["operation"].as_str().unwrap() {
    //                     "fetch_one" => {
    //                         let input = operation["input"].clone();
    //                         let input = from_value(input).map_err(|e| {
    //                             json!({
    //                                 "error": "failed to deserialize input",
    //                                 "error_message": e.to_string()
    //                             })
    //                         })?;
    //                         let output = self.fetch_one(input).await;
    //                         let output = to_value(output).map_err(|e| {
    //                             json!({
    //                                 "error": "failed to serialize output",
    //                                 "error_message": e.to_string()
    //                             })
    //                         })?;
    //                         return Ok(output);
    //                     }
    //                     _ => {
    //                         return Err(json!({
    //                             "error": "unknown operation",
    //                             "operation": operation["operation"].as_str().unwrap()
    //                         }));
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    pub mod json_link_fetch_many_extention {
        use core::fmt;
        use std::{any::Any, sync::Arc};

        use serde::Serialize;
        use sqlx::Database;

        use crate::{
            database_extention::DatabaseExt,
            from_row::{FromRowAlias, FromRowData, FromRowError, RowPreAliased, RowTwoAliased},
            operations::{
                Operation, OperationOutput, boxed_operation::BoxedOperation,
                fetch_many_cursor_multi_col::LinkFetch,
            },
            query_builder::{
                ManyBoxedExpressions, ManyExpressions,
                functional_expr::{BoxedExpression, ManyFlat},
            },
        };

        pub trait JsonLinkFetchMany<S>: Send {
            fn join(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;
            fn wheres_expression(&self) -> Box<dyn ManyBoxedExpressions<S> + Send>;

            fn select_items(&self) -> Box<dyn Any + Send>;
            fn select_items_expression(
                &self,
                alias: &'static str,
                si: &Box<dyn Any + Send>,
            ) -> Box<dyn ManyBoxedExpressions<S> + Send>;
            fn select_items_pre_alias<'r>(
                &self,
                si: &Box<dyn Any + Send>,
                row: RowPreAliased<'r, S::Row>,
            ) -> Result<Box<dyn Any + Send>, FromRowError>
            where
                S: Database;
            fn select_items_no_alias<'r>(
                &self,
                si: &Box<dyn Any + Send>,
                row: &'r S::Row,
            ) -> Result<Box<dyn Any + Send>, FromRowError>
            where
                S: Database;
            fn select_items_two_alias<'r>(
                &self,
                si: &Box<dyn Any + Send>,
                row: RowTwoAliased<'r, S::Row>,
            ) -> Result<Box<dyn Any + Send>, FromRowError>
            where
                S: Database;
            fn post_select_input_init_2(&self) -> Box<dyn Any + Send>;
            fn post_select_each_2(&self, item: &Box<dyn Any + Send>, poi: &mut Box<dyn Any + Send>);
            fn post_select_2(
                &self,
                input: Box<dyn Any + Send>,
            ) -> Box<dyn BoxedOperation<S> + Send>;
            fn take_2(
                &self,
                item: Box<dyn Any + Send>,
                op: &mut Box<dyn Any + Send>,
            ) -> serde_json::Value;
        }

        impl<S, T> JsonLinkFetchMany<S> for T
        where
            S: Send,
            S: DatabaseExt,
            T: Send + Sync + 'static,
            T: LinkFetchMany,
            T::SelectItems: 'static,
            T::SelectItems: Clone,
            T::SelectItems:
                Send + for<'r> FromRowAlias<'r, S::Row, RData: fmt::Debug + Send + 'static>,
            T::SelectItems: Aliased<Aliased: Send + for<'q> ManyExpressions<'q, S>>,
            T::Join: 'static + Send + ManyBoxedExpressions<S>,
            T::Wheres: 'static + Send + ManyBoxedExpressions<S>,
            T::PostOperation: 'static + Send + Operation<S>,
            T::Output: Serialize,
            T::PostOperationInput: Send + 'static,
        {
            fn take_2(
                &self,
                item: Box<dyn Any + Send>,
                op: &mut Box<dyn Any + Send>,
            ) -> serde_json::Value {
                let item = *item
                    .downcast::<<T::SelectItems as FromRowData>::RData>()
                    .unwrap();
                let op = op
                    .downcast_mut::<<T::PostOperation as OperationOutput>::Output>()
                    .unwrap();
                let o = self.take(item, op);
                serde_json::to_value(o).expect("bug: serializing should not fail")
            }
            fn post_select_2(
                &self,
                input: Box<dyn Any + Send>,
            ) -> Box<dyn BoxedOperation<S> + Send> {
                let s = *input.downcast::<T::PostOperationInput>().unwrap();
                let s = self.post_select(s);
                Box::new(s)
            }
            fn join(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
                Box::new(self.non_duplicating_join())
            }
            fn wheres_expression(&self) -> Box<dyn ManyBoxedExpressions<S> + Send> {
                Box::new(self.wheres())
            }
            fn select_items(&self) -> Box<dyn Any + Send> {
                let select_items: T::SelectItems = self.non_aggregating_select_items();
                Box::new(select_items)
            }
            fn select_items_expression(
                &self,
                alias: &'static str,
                si: &Box<dyn Any + Send>,
            ) -> Box<dyn ManyBoxedExpressions<S> + Send> {
                let downcasted: &T::SelectItems = si.downcast_ref::<T::SelectItems>().unwrap();
                let cloned = downcasted.clone();
                Box::new(cloned.str_aliased(alias))
            }

            fn select_items_pre_alias<'r>(
                &self,
                si: &Box<dyn Any + Send>,
                row: RowPreAliased<'r, <S>::Row>,
            ) -> Result<Box<dyn Any + Send>, FromRowError>
            where
                S: Database,
            {
                let downcasted: &T::SelectItems = si.downcast_ref::<T::SelectItems>().unwrap();
                let rdata: <T::SelectItems as FromRowData>::RData = downcasted.pre_alias(row)?;
                Ok(Box::new(rdata))
            }

            fn select_items_no_alias<'r>(
                &self,
                si: &Box<dyn Any + Send>,
                row: &'r S::Row,
            ) -> Result<Box<dyn Any + Send>, FromRowError>
            where
                S: Database,
            {
                let downcasted: &T::SelectItems = si.downcast_ref::<T::SelectItems>().unwrap();
                let rdata: <T::SelectItems as FromRowData>::RData = downcasted.no_alias(row)?;
                Ok(Box::new(rdata))
            }

            fn select_items_two_alias<'r>(
                &self,
                si: &Box<dyn Any + Send>,
                row: RowTwoAliased<'r, S::Row>,
            ) -> Result<Box<dyn Any + Send>, FromRowError>
            where
                S: Database,
            {
                let downcasted: &T::SelectItems = si.downcast_ref::<T::SelectItems>().unwrap();
                let rdata: <T::SelectItems as FromRowData>::RData = downcasted.two_alias(row)?;
                Ok(Box::new(rdata))
            }

            fn post_select_input_init_2(&self) -> Box<dyn Any + Send> {
                let s: T::PostOperationInput = self.post_operation_input_init();
                Box::new(s)
            }
            fn post_select_each_2(
                &self,
                item: &Box<dyn Any + Send>,
                poi: &mut Box<dyn Any + Send>,
            ) {
                let item = item
                    .downcast_ref::<<T::SelectItems as FromRowData>::RData>()
                    .unwrap();
                let poi = poi.downcast_mut::<T::PostOperationInput>().unwrap();
                self.post_select_each(item, poi);
            }
        }

        pub struct JsonLinkSelectItems<'a, S> {
            pub this: Arc<dyn JsonLinkFetchMany<S> + Send + Sync + 'a>,
            pub select_items: Box<dyn Any + Send>,
        }

        impl<'a, S: Database> StrAliased for JsonLinkSelectItems<'a, S> {
            type StrAliased = Box<dyn ManyBoxedExpressions<S> + Send>;

            fn str_aliased(&self, alias: &'static str) -> Self::StrAliased {
                self.this.select_items_expression(alias, &self.select_items)
            }
        }

        impl<'a, S> StrAliased for Vec<JsonLinkSelectItems<'a, S>> {
            type StrAliased = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;
            fn str_aliased(&self, alias: &'static str) -> Self::StrAliased {
                ManyFlat(
                    self.iter()
                        .map(|each| each.this.select_items_expression(alias, &each.select_items))
                        .collect::<Vec<_>>(),
                )
            }
        }

        impl<'a, S> FromRowData for Vec<JsonLinkSelectItems<'a, S>> {
            type RData = Vec<Box<dyn Any + Send>>;
        }

        impl<'r, 'a, S: Database> FromRowAlias<'r, S::Row> for Vec<JsonLinkSelectItems<'a, S>> {
            fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
                Ok(self
                    .iter()
                    .map(|each| each.no_alias(row))
                    .collect::<Result<Vec<_>, FromRowError>>()?)
            }

            fn pre_alias(&self, row: RowPreAliased<'r, S::Row>) -> Result<Self::RData, FromRowError>
            where
                S::Row: sqlx::Row,
            {
                Ok(self
                    .iter()
                    .map(|each| each.pre_alias(row.clone()))
                    .collect::<Result<Vec<_>, FromRowError>>()?)
            }

            fn post_alias(
                &self,
                row: crate::from_row::RowPostAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: sqlx::Row,
            {
                let _ = row;
                panic!("in the process of deprecating this method")
            }

            fn two_alias(&self, row: RowTwoAliased<'r, S::Row>) -> Result<Self::RData, FromRowError>
            where
                S::Row: sqlx::Row,
            {
                Ok(self
                    .iter()
                    .map(|each| each.two_alias(row.clone()))
                    .collect::<Result<Vec<_>, FromRowError>>()?)
            }
        }

        impl<'a, S> FromRowData for JsonLinkSelectItems<'a, S> {
            type RData = Box<dyn Any + Send>;
        }
        impl<'r, 'a, S: Database> FromRowAlias<'r, S::Row> for JsonLinkSelectItems<'a, S> {
            fn no_alias(&self, row: &'r S::Row) -> Result<Self::RData, FromRowError> {
                self.this.select_items_no_alias(&self.select_items, row)
            }

            fn pre_alias(&self, row: RowPreAliased<'r, S::Row>) -> Result<Self::RData, FromRowError>
            where
                S::Row: sqlx::Row,
            {
                self.this.select_items_pre_alias(&self.select_items, row)
            }

            fn post_alias(
                &self,
                row: crate::from_row::RowPostAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: sqlx::Row,
            {
                let _ = row;
                panic!("in the process of deprecating this method")
            }

            fn two_alias(
                &self,
                row: crate::from_row::RowTwoAliased<'r, S::Row>,
            ) -> Result<Self::RData, FromRowError>
            where
                S::Row: sqlx::Row,
            {
                self.this.select_items_two_alias(&self.select_items, row)
            }
        }

        #[allow(unused)]
        impl<'a, S: Database> LinkFetchMany for Vec<Arc<dyn JsonLinkFetchMany<S> + Send + Sync + 'a>>
        where
            Vec<JsonLinkSelectItems<'a, S>>: FromRowData<RData = Vec<Box<dyn Any + Send>>>,
            Vec<Box<dyn BoxedOperation<S> + Send>>:
                OperationOutput<Output = Vec<Box<dyn Any + Send>>>,
        {
            type Output = Vec<serde_json::Value>;

            type SelectItems = Vec<JsonLinkSelectItems<'a, S>>;

            fn non_aggregating_select_items(&self) -> Self::SelectItems {
                todo!("how to ensure using TwoAlias");
                self.iter()
                    .map(|each| each.non_aggregating_select_items())
                    .collect()
            }

            type Join = ManyFlat<Vec<Box<dyn BoxedExpression<S> + Send>>>;

            fn non_duplicating_join(&self) -> Self::Join {
                ManyFlat(self.iter().map(|each| each.join()).collect::<Vec<_>>())
            }

            type Wheres = ManyFlat<Vec<Box<dyn ManyBoxedExpressions<S> + Send>>>;

            fn wheres(&self) -> Self::Wheres {
                ManyFlat(
                    self.iter()
                        .map(|each| each.wheres_expression())
                        .collect::<Vec<_>>(),
                )
            }

            type PostOperationInput = Vec<Box<dyn Any + Send>>;

            fn post_operation_input_init(&self) -> Self::PostOperationInput {
                self.iter()
                    .map(|each| each.post_operation_input_init())
                    .collect()
            }

            type PostOperation = Vec<Box<dyn BoxedOperation<S> + Send>>;

            fn post_select(&self, input: Self::PostOperationInput) -> Self::PostOperation
            where
                Self::SelectItems: FromRowData,
            {
                self.iter()
                    .zip(input)
                    .map(|(each, input)| each.post_select(input))
                    .collect()
            }

            fn post_select_each(
                &self,
                item: &<Self::SelectItems as FromRowData>::RData,
                poi: &mut Self::PostOperationInput,
            ) where
                Self::SelectItems: FromRowData,
            {
                for (index, each) in self.iter().enumerate() {
                    let item = item.get(index).unwrap();
                    let poi = poi.get_mut(index).unwrap();
                    each.post_select_each(item, poi);
                }
            }

            fn take(
                &self,
                item: <Self::SelectItems as FromRowData>::RData,
                op: &mut <Self::PostOperation as OperationOutput>::Output,
            ) -> Self::Output
            where
                Self::SelectItems: FromRowData,
                Self::PostOperation: OperationOutput,
            {
                self.iter()
                    .zip(item)
                    .zip(op)
                    .map(|((each, item), op)| each.take(item, op))
                    .collect()
            }
        }
        impl<'a, S: Database> LinkFetchMany for Arc<dyn JsonLinkFetchMany<S> + Send + Sync + 'a>
        where
            JsonLinkSelectItems<'a, S>: FromRowData<RData = Box<dyn Any + Send>>,
            Box<dyn BoxedOperation<S> + Send>: OperationOutput<Output = Box<dyn Any + Send>>,
        {
            type Output = serde_json::Value;

            type SelectItems = JsonLinkSelectItems<'a, S>;

            fn non_aggregating_select_items(&self) -> Self::SelectItems {
                JsonLinkSelectItems {
                    this: self.clone(),
                    select_items: self.select_items(),
                }
            }

            type Join = Box<dyn BoxedExpression<S> + Send>;

            fn non_duplicating_join(&self) -> Self::Join {
                self.join()
            }

            type Wheres = Box<dyn ManyBoxedExpressions<S> + Send>;

            fn wheres(&self) -> Self::Wheres {
                self.wheres_expression()
            }

            type PostOperationInput = Box<dyn Any + Send>;
            type PostOperation = Box<dyn BoxedOperation<S> + Send>;

            fn post_select(&self, input: Self::PostOperationInput) -> Self::PostOperation
            where
                Self::SelectItems: crate::from_row::FromRowData,
            {
                self.post_select_2(input)
            }

            fn take(
                &self,
                item: <Self::SelectItems as FromRowData>::RData,
                op: &mut <Self::PostOperation as OperationOutput>::Output,
            ) -> Self::Output
            where
                Self::PostOperation: OperationOutput,
            {
                let item: Box<dyn Any + Send> = item; // <T::SelectItems as FromRowData>::RData
                let op: &mut Box<dyn Any + Send> = op; // <Self::PostOperation as OperationOutput>::Output
                let o = self.take_2(item, op);

                o
            }

            fn post_operation_input_init(&self) -> Self::PostOperationInput {
                self.post_select_input_init_2()
            }

            fn post_select_each(
                &self,
                item: &<Self::SelectItems as FromRowData>::RData,
                poi: &mut Self::PostOperationInput,
            ) where
                Self::SelectItems: FromRowData,
            {
                self.post_select_each_2(item, poi);
            }
        }
    }
}
