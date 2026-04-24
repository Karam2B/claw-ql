pub trait Collection {
    fn table_name(&self) -> &str;
    fn table_name_lower_case(&self) -> &str;
    type Data;
    type Id: CollectionId;
    fn id(&self) -> Self::Id;
}

/// dyn-compatable parts of Collection
pub trait CollectionBasic {
    fn table_name(&self) -> &str;
    fn table_name_lower_case(&self) -> &str;
}

impl<T> CollectionBasic for T
where
    T: Collection,
{
    fn table_name(&self) -> &str {
        self.table_name()
    }
    fn table_name_lower_case(&self) -> &str {
        self.table_name_lower_case()
    }
}

pub trait Member {
    fn name(&self) -> &str;
    type Data;
    type CollectionHandler;
}

pub trait HasHandler {
    type Handler;
}

pub trait CollectionId {
    type IdData;
}

pub trait SingleColumnId: CollectionId + AsRef<str> {}

#[derive(Clone, Debug)]
pub struct SingleIncremintalInt<T>(pub T);

pub(crate) mod impl_id {

    use sqlx::Sqlite;

    use crate::{
        collections::{CollectionId, SingleColumnId, SingleIncremintalInt},
        expressions::single_col_expressions::{AliasedCol, ScopedCol, UpdatingCol},
        extentions::common_expressions::{
            Aliased, Identifier, MigrateExpression, OnUpdate, Scoped,
        },
        query_builder::{Expression, OpExpression, SanitizeMany, StatementBuilder},
        update_mod::Update,
    };

    impl<T> AsRef<str> for SingleIncremintalInt<T> {
        fn as_ref(&self) -> &str {
            "id"
        }
    }

    impl<T> CollectionId for SingleIncremintalInt<T> {
        type IdData = i64;
    }

    impl<T> SingleColumnId for SingleIncremintalInt<T> {}

    impl Aliased for SingleIncremintalInt<&'static str> {
        type Aliased =
            AliasedCol<&'static str, &'static str, SanitizeMany<(&'static str, &'static str)>>;
        fn aliased(&self, alias: &'static str) -> Self::Aliased {
            AliasedCol {
                table: self.0,
                col: "id",
                alias: SanitizeMany((alias, "id")),
            }
        }
        type NumAliased = AliasedCol<
            &'static str,
            &'static str,
            SanitizeMany<(&'static str, usize, &'static str)>,
        >;
        fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
            AliasedCol {
                table: self.0,
                col: "id",
                alias: SanitizeMany((alias, num, "id")),
            }
        }
    }

    impl Aliased for SingleIncremintalInt<String> {
        type Aliased = AliasedCol<String, &'static str, SanitizeMany<(&'static str, &'static str)>>;
        fn aliased(&self, alias: &'static str) -> Self::Aliased {
            AliasedCol {
                table: self.0.clone(),
                col: "id",
                alias: SanitizeMany((alias, "id")),
            }
        }
        type NumAliased =
            AliasedCol<String, &'static str, SanitizeMany<(&'static str, usize, &'static str)>>;
        fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
            AliasedCol {
                table: self.0.clone(),
                col: "id",
                alias: SanitizeMany((alias, num, "id")),
            }
        }
    }

    impl Scoped for SingleIncremintalInt<&'static str> {
        type Scoped = ScopedCol<&'static str, &'static str>;
        fn scoped(&self) -> Self::Scoped {
            ScopedCol {
                table: self.0,
                col: "id",
            }
        }
    }

    impl Scoped for SingleIncremintalInt<String> {
        type Scoped = ScopedCol<String, &'static str>;
        fn scoped(&self) -> Self::Scoped {
            ScopedCol {
                table: self.0.clone(),
                col: "id",
            }
        }
    }

    impl<T> Identifier for SingleIncremintalInt<T> {
        type Identifier = &'static str;
        fn identifier(&self) -> Self::Identifier {
            "id"
        }
    }

    impl OnUpdate for SingleIncremintalInt<&'static str> {
        type UpdateInput = Update<i64>;
        type UpdateExpression = UpdatingCol<&'static str, i64>;
        fn validate_on_update(&self, input: Self::UpdateInput) -> Self::UpdateExpression {
            UpdatingCol {
                col: self.identifier(),
                set: input,
            }
        }
    }

    impl OnUpdate for SingleIncremintalInt<String> {
        type UpdateInput = Update<i64>;
        type UpdateExpression = UpdatingCol<String, i64>;
        fn validate_on_update(&self, input: Self::UpdateInput) -> Self::UpdateExpression {
            UpdatingCol {
                col: self.0.clone(),
                set: input,
            }
        }
    }

    pub struct IdMigration;

    impl<T> MigrateExpression for SingleIncremintalInt<T> {
        type MigrateExpression = IdMigration;
        fn migrate_expression(&self) -> Self::MigrateExpression {
            IdMigration
        }
    }

    impl OpExpression for IdMigration {}
    impl<'q> Expression<'q, Sqlite> for IdMigration {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>) {
            ctx.syntax(&"\"id\" INTEGER PRIMARY KEY AUTOINCREMENT");
        }
    }

    mod from_row_impls {
        use sqlx::{ColumnIndex, Decode, Row, Type};

        use super::SingleIncremintalInt;
        use crate::from_row::{
            FromRowAlias, FromRowData, FromRowError, RowPostAliased, RowPreAliased, RowTwoAliased,
            TryFromRowAlias,
        };

        impl<T> FromRowData for SingleIncremintalInt<T> {
            type RData = i64;
        }

        impl<'r, R: Row, T> FromRowAlias<'r, R> for SingleIncremintalInt<T>
        where
            R: Row + 'r,
            i64: Type<R::Database> + Decode<'r, R::Database>,
            for<'a> &'a str: ColumnIndex<R>,
        {
            fn no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError> {
                Ok(row.try_get("id")?)
            }
            fn pre_alias(&self, row: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError> {
                Ok(row.try_get("id")?)
            }
            fn post_alias(&self, row: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError> {
                Ok(row.try_get("id")?)
            }
            fn two_alias(&self, row: RowTwoAliased<'r, R>) -> Result<Self::RData, FromRowError> {
                Ok(row.try_get("id")?)
            }
        }

        // T is infered to be Option<i64>
        impl<'r, R: Row, T> TryFromRowAlias<'r, R> for SingleIncremintalInt<T>
        where
            R: Row + 'r,
            i64: Type<R::Database> + Decode<'r, R::Database>,
            for<'a> &'a str: ColumnIndex<R>,
        {
            fn try_no_alias(&self, row: &'r R) -> Result<Option<Self::RData>, FromRowError> {
                Ok(row.get("id"))
            }

            fn try_pre_alias(
                &self,
                row: RowPreAliased<'r, R>,
            ) -> Result<Option<Self::RData>, FromRowError>
            where
                R: Row,
            {
                Ok(row.get("id"))
            }

            fn try_two_alias(
                &self,
                row: RowTwoAliased<'r, R>,
            ) -> Result<Option<Self::RData>, FromRowError>
            where
                R: Row,
            {
                Ok(row.get("id"))
            }

            fn try_post_alias(
                &self,
                row: RowPostAliased<'r, R>,
            ) -> Result<Option<Self::RData>, FromRowError>
            where
                R: Row,
            {
                Ok(row.get("id"))
            }
        }
    }
}
