pub struct Timestamp<C> {
    pub collection: C,
}

pub struct TimestampOutput {
    pub created_at: String,
    pub updated_at: String,
}

pub mod expressions {

    pub mod sql_default {
        use crate::{
            database_extention::DatabaseExt,
            query_builder::{Expression, OpExpression, StatementBuilder},
        };

        pub struct SqlDefault<T> {
            pub value: T,
        }

        impl<T> OpExpression for SqlDefault<T> {}
        impl<'q, S, T> Expression<'q, S> for SqlDefault<T>
        where
            T: 'q + Expression<'q, S>,
            S: DatabaseExt,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>)
            where
                S: DatabaseExt,
            {
                ctx.syntax("DEFAULT ");
                self.value.expression(ctx);
            }
        }
    }

    pub mod current_timestamp {
        use crate::query_builder::OpExpression;

        pub struct CurrentTimestamp;

        impl OpExpression for CurrentTimestamp {}

        mod for_sqlite {
            use sqlx::Sqlite;

            use crate::{
                database_extention::DatabaseExt,
                links::timestamp::expressions::current_timestamp::CurrentTimestamp,
                query_builder::{Expression, StatementBuilder},
            };

            impl<'q> Expression<'q, Sqlite> for CurrentTimestamp {
                fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
                where
                    Sqlite: DatabaseExt,
                {
                    ctx.syntax("CURRENT_TIMESTAMP");
                }
            }
        }
    }

    pub mod verbatim_statement {
        use crate::{
            database_extention::DatabaseExt,
            query_builder::{Expression, OpExpression, StatementBuilder},
        };

        pub struct VerbatimStatement<S> {
            pub verbatim: String,
            pub for_db: S,
        }

        pub struct VerbatimForAnyDb;

        impl<S> OpExpression for VerbatimStatement<S> {}
        impl<'q, S> Expression<'q, S> for VerbatimStatement<S>
        where
            S: DatabaseExt,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
                ctx.stmt.push_str(&self.verbatim);
            }
        }

        impl<'q, S> Expression<'q, S> for VerbatimStatement<VerbatimForAnyDb>
        where
            S: DatabaseExt,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
                ctx.stmt.push_str(&self.verbatim);
            }
        }
    }

    pub mod create_trigger {
        use crate::query_builder::{ManyExpressions, OpExpression};

        pub struct CreateTrigger<
            TriggerName,
            Lifetime,
            OperationName,
            OnTable,
            WhenExpression,
            Statements,
        > {
            pub trigger_name: TriggerName,
            pub temp: bool,
            pub if_not_exists: bool,
            pub lifetime: Lifetime,
            pub operation_name: OperationName,
            pub on_table: OnTable,
            pub for_each_row: bool,
            pub when_expression: WhenExpression,
            pub statements: Statements,
        }

        mod for_sqlite {
            use sqlx::Sqlite;

            use crate::{
                links::timestamp::expressions::create_trigger::CreateTrigger,
                query_builder::{Expression, ManyExpressions, OpExpression, PossibleExpression},
            };

            impl<Tn, Lt, On, Ont, We, St> OpExpression for CreateTrigger<Tn, Lt, On, Ont, We, St> {}
            impl<'q, TriggerName, Lifetime, OperationName, OnTable, WhenExpression, Statements>
                Expression<'q, Sqlite>
                for CreateTrigger<
                    TriggerName,
                    Lifetime,
                    OperationName,
                    OnTable,
                    WhenExpression,
                    Statements,
                >
            where
                TriggerName: 'q + Expression<'q, Sqlite>,
                Lifetime: 'q + PossibleExpression<'q, Sqlite>,
                OperationName: 'q + Expression<'q, Sqlite>,
                OnTable: 'q + Expression<'q, Sqlite>,
                WhenExpression: 'q + PossibleExpression<'q, Sqlite>,
                Statements: 'q + ManyExpressions<'q, Sqlite>,
            {
                fn expression(self, ctx: &mut crate::query_builder::StatementBuilder<'q, Sqlite>) {
                    ctx.syntax("CREATE ");
                    if self.temp {
                        ctx.syntax("TEMP ");
                    }
                    if self.if_not_exists {
                        ctx.syntax("IF NOT EXISTS ");
                    }
                    ctx.syntax("TRIGGER ");
                    self.trigger_name.expression(ctx);
                    ctx.syntax(" ");
                    self.lifetime.expression(ctx);
                    ctx.syntax(" ");
                    self.operation_name.expression(ctx);
                    ctx.syntax(" ON ");
                    self.on_table.expression(ctx);
                    ctx.syntax(" ");
                    if self.for_each_row {
                        ctx.syntax("FOR EACH ROW ");
                    }
                    ctx.syntax(" ");
                    if self.when_expression.is_op() {
                        ctx.syntax("WHEN ");
                        self.when_expression.expression(ctx);
                    }
                    ctx.syntax(" BEGIN ");
                    self.statements.expression("", ";", ctx);
                    ctx.syntax(" END;");
                }
            }
        }

        pub struct TriggerLifetimeBefore;

        impl OpExpression for TriggerLifetimeBefore {}

        const _: () = {
            use sqlx::Sqlite;

            use crate::{
                links::timestamp::expressions::create_trigger::TriggerLifetimeBefore,
                query_builder::{Expression, StatementBuilder},
            };

            impl<'q> Expression<'q, Sqlite> for TriggerLifetimeBefore {
                fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>) {
                    ctx.syntax("BEFORE");
                }
            }
        };

        pub struct TriggerLifetimeAfter;

        impl OpExpression for TriggerLifetimeAfter {}

        const _: () = {
            use sqlx::Sqlite;

            use crate::{
                links::timestamp::expressions::create_trigger::TriggerLifetimeAfter,
                query_builder::{Expression, StatementBuilder},
            };

            impl<'q> Expression<'q, Sqlite> for TriggerLifetimeAfter {
                fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>) {
                    ctx.syntax("AFTER");
                }
            }
        };

        pub struct TriggerLifetimeInsteadOf;

        impl OpExpression for TriggerLifetimeInsteadOf {}

        const _: () = {
            use sqlx::Sqlite;

            use crate::{
                links::timestamp::expressions::create_trigger::TriggerLifetimeInsteadOf,
                query_builder::{Expression, StatementBuilder},
            };
            impl<'q> Expression<'q, Sqlite> for TriggerLifetimeInsteadOf {
                fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>) {
                    ctx.syntax("INSTEAD OF");
                }
            }
        };

        pub struct TriggerOperationNameInsert;

        impl OpExpression for TriggerOperationNameInsert {}

        const _: () = {
            use sqlx::Sqlite;

            use crate::{
                links::timestamp::expressions::create_trigger::TriggerOperationNameInsert,
                query_builder::{Expression, StatementBuilder},
            };
            impl<'q> Expression<'q, Sqlite> for TriggerOperationNameInsert {
                fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>) {
                    ctx.syntax("INSERT");
                }
            }
        };

        pub struct TriggerOperationNameUpdate<OfColumns>(pub OfColumns);

        impl<OfColumns> OpExpression for TriggerOperationNameUpdate<OfColumns> {}

        const _: () = {
            use sqlx::Sqlite;

            use crate::{
                links::timestamp::expressions::create_trigger::TriggerOperationNameUpdate,
                query_builder::{Expression, StatementBuilder},
            };

            impl<'q, OfColumns> Expression<'q, Sqlite> for TriggerOperationNameUpdate<OfColumns>
            where
                OfColumns: 'q + ManyExpressions<'q, Sqlite>,
            {
                fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>) {
                    ctx.syntax("UPDATE");

                    self.0.expression(" OF ", ", ", ctx);
                }
            }
        };
    }
}

mod impl_on_migrate {
    use std::marker::PhantomData;

    use sqlx::Sqlite;

    use crate::{
        collections::Collection,
        expressions::col_def,
        extentions::common_expressions::TableNameExpression,
        links::timestamp::{
            Timestamp,
            expressions::{
                create_trigger::{CreateTrigger, TriggerLifetimeAfter, TriggerOperationNameUpdate},
                current_timestamp::CurrentTimestamp,
                sql_default::SqlDefault,
                verbatim_statement::VerbatimStatement,
            },
        },
        on_migrate::OnMigrate,
        statements::AddColumn,
    };

    impl<C> OnMigrate for Timestamp<C>
    where
        C: Collection,
        C: TableNameExpression,
    {
        type Statements = (
            AddColumn<
                C::TableNameExpression,
                col_def<&'static str, String, SqlDefault<CurrentTimestamp>>,
            >,
            AddColumn<
                C::TableNameExpression,
                col_def<&'static str, String, SqlDefault<CurrentTimestamp>>,
            >,
            CreateTrigger<
                &'static str,
                TriggerLifetimeAfter,
                TriggerOperationNameUpdate<()>,
                C::TableNameExpression,
                (),
                VerbatimStatement<Sqlite>,
            >,
        );
        fn statments(&self) -> Self::Statements {
            let trigger = CreateTrigger {
                trigger_name: "update_timestamp",
                temp: false,
                if_not_exists: false,
                lifetime: TriggerLifetimeAfter,
                operation_name: TriggerOperationNameUpdate(()),
                on_table: self.collection.table_name_expression(),
                when_expression: (),
                for_each_row: false,
                statements: VerbatimStatement {
                    verbatim: String::from(
                        "UPDATE {table} SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;",
                    ),
                    for_db: Sqlite,
                },
            };
            (
                AddColumn {
                    table: self.collection.table_name_expression(),
                    col_def: col_def {
                        name: "created_at",
                        ty: PhantomData,
                        constraints: SqlDefault {
                            value: CurrentTimestamp,
                        },
                    },
                },
                AddColumn {
                    table: self.collection.table_name_expression(),
                    col_def: col_def {
                        name: "updated_at",
                        ty: PhantomData,
                        constraints: SqlDefault {
                            value: CurrentTimestamp,
                        },
                    },
                },
                trigger,
            )
        }
    }

    #[cfg(test)]
    mod test {
        use sqlx::Sqlite;

        use crate::{
            links::timestamp::Timestamp,
            on_migrate::OnMigrate,
            prelude::sql::ManyPossible,
            query_builder::{StatementBuilder, functional_expr::ManyImplExpression},
            test_module,
        };

        #[test]
        fn on_migrate_for_sqlite() {
            let many = ManyImplExpression {
                item: ManyPossible(
                    Timestamp {
                        collection: test_module::todo,
                    }
                    .statments(),
                ),
                start: "",
                join: ";",
            };
            let s = StatementBuilder::<Sqlite>::new(many);

            pretty_assertions::assert_eq!(
                s.stmt(),
                "CREATE TRIGGER update_timestamp AFTER UPDATE ON todo BEGIN UPDATE todo SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;",
            );
        }
    }
}

mod impl_fetch_many {
    use sqlx::{ColumnIndex, Decode, Row, Type};
    use tokio::time::Timeout;

    use crate::{
        from_row::{FromRowAlias, FromRowData},
        links::timestamp::TimestampOutput,
        operations::fetch_many::LinkFetchMany,
    };

    pub struct TimestampSelectItems;

    impl FromRowData for TimestampSelectItems {
        type RData = TimestampOutput;
    }

    impl<'r, R> FromRowAlias<'r, R> for TimestampSelectItems
    where
        R: Row,
        for<'s> &'s str: ColumnIndex<R>,
        String: for<'d> Decode<'d, R::Database> + Type<R::Database>,
    {
        fn no_alias(&self, row: &'r R) -> Result<Self::RData, crate::from_row::FromRowError> {
            Ok(TimestampOutput {
                created_at: row.get("crated_at"),
                updated_at: row.get("update_at"),
            })
        }

        fn pre_alias(
            &self,
            row: crate::from_row::RowPreAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            Ok(TimestampOutput {
                created_at: row.get("crated_at"),
                updated_at: row.get("update_at"),
            })
        }

        fn post_alias(
            &self,
            row: crate::from_row::RowPostAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            Ok(TimestampOutput {
                created_at: row.get("crated_at"),
                updated_at: row.get("update_at"),
            })
        }

        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            Ok(TimestampOutput {
                created_at: row.get("crated_at"),
                updated_at: row.get("update_at"),
            })
        }
    }

    impl<C> LinkFetchMany for Timeout<C> {
        type Output = TimestampOutput;

        type SelectItems = TimestampSelectItems;

        fn non_aggregating_select_items(&self) -> Self::SelectItems {
            TimestampSelectItems
        }

        type Join = ();

        fn non_duplicating_join(&self) -> Self::Join {}

        type Wheres = ();

        fn wheres(&self) -> Self::Wheres {}

        type PostOperationInput = ();

        fn post_operation_input_init(&self) -> Self::PostOperationInput {}

        type PostOperation = ();

        fn post_select(&self, _: Self::PostOperationInput) -> Self::PostOperation
        where
            Self::SelectItems: crate::from_row::FromRowData,
        {
        }

        fn post_select_each(
            &self,
            _: &<Self::SelectItems as crate::from_row::FromRowData>::RData,
            _: &mut Self::PostOperationInput,
        ) where
            Self::SelectItems: crate::from_row::FromRowData,
        {
        }

        fn take(
            &self,
            item: <Self::SelectItems as crate::from_row::FromRowData>::RData,
            _: &mut <Self::PostOperation as crate::operations::OperationOutput>::Output,
        ) -> Self::Output
        where
            Self::SelectItems: crate::from_row::FromRowData,
            Self::PostOperation: crate::operations::OperationOutput,
        {
            item
        }
    }
}
