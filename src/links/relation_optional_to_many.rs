#![allow(unexpected_cfgs)]

use crate::links::{LinkedToBase, LinkedViaId};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct OptionalToMany<Id, F, T> {
    pub foriegn_key: Id,
    pub from: F,
    pub to: T,
}

impl<Id, F, T> LinkedViaId for OptionalToMany<Id, F, T> {}

impl<Id, F, T> LinkedToBase for OptionalToMany<Id, F, T> {
    type Base = F;
}

mod impl_on_migrate {
    use std::marker::PhantomData;

    use crate::{
        collections::{Collection, SingleColumnId},
        expressions::{
            ColumnDefinition, foriegn_key, on_delete_set_null,
            standard_naming_conventions::ForeignKeyName,
        },
        extentions::common_expressions::TableNameExpression,
        links::relation_optional_to_many::OptionalToMany,
        on_migrate::OnMigrate,
        query_builder::functional_expr::ManyPossible,
        statements::AddColumn,
    };

    impl<Key, F, T> OnMigrate for OptionalToMany<Key, F, T>
    where
        Key: AsRef<str>,
        F: Collection + Clone,
        T: Collection<Id: SingleColumnId> + Clone,
        F: TableNameExpression,
        T: TableNameExpression,
        Key: Clone,
    {
        type Statements = AddColumn<
            F::TableNameExpression,
            ColumnDefinition<
                ForeignKeyName<Key, T::TableNameExpression>,
                Option<i64>,
                foriegn_key<ManyPossible<(on_delete_set_null,)>>,
            >,
        >;
        fn statments(&self) -> Self::Statements {
            AddColumn {
                table: self.from.table_name_expression(),
                col_def: ColumnDefinition {
                    name: ForeignKeyName {
                        key: self.foriegn_key.clone(),
                        to: self.to.table_name_expression(),
                    },
                    ty: PhantomData,
                    constraints: foriegn_key {
                        references_table: self.to.table_name().to_string(),
                        references_col: self.to.id().as_ref().to_string(),
                        ons: ManyPossible((on_delete_set_null,)),
                    },
                },
            }
        }
    }
}

#[claw_ql_macros::skip]
// to be refactored
mod impl_fetch_one {
    use sqlx::{ColumnIndex, Database, Row, Type};

    use crate::{
        collections::{Collection, SingleIncremintalInt},
        expressions::left_join,
        extentions::Members,
        from_row::{FromRowAlias, RowTwoAliased},
        links::relation_optional_to_many::OptionalToMany,
        operations::{CollectionOutput, Operation},
    };

    impl<Id, S, F, T> LinkFetchOne<S> for OptionalToMany<Id, F, T>
    where
        Id: AsRef<str>,
        T: Collection<Id: SingleColumnId> + Members,
        T: for<'r> FromRowAlias<'r, <S as Database>::Row, FromRowData = T::Data>,
        F: Collection,
        S: Database,
        i64: for<'q> sqlx::Decode<'q, S> + Type<S>,
        for<'q> &'q str: ColumnIndex<S::Row>,
    {
        type Joins = left_join;

        type Wheres = ();

        fn non_aggregating_select_items(&self) -> Vec<link_select_item<String, String>> {
            let mut base = vec![link_select_item::new(
                self.to.table_name().to_string(),
                "id".to_string(),
            )];

            base.extend(
                self.to
                    .members_names()
                    .into_iter()
                    .map(|e| link_select_item::new(self.to.table_name().to_string(), e)),
            );

            base
        }
        fn non_duplicating_joins(&self) -> Self::Joins {
            left_join {
                ft: self.to.table_name().to_string(),
                fc: "id".to_string(),
                lt: self.from.table_name().to_string(),
                lc: format!(
                    "fk_{}_{}_{}",
                    self.from.table_name_lower_case(),
                    self.to.table_name_lower_case(),
                    self.foriegn_key.as_ref()
                ),
            }
        }
        fn wheres(&self) -> Self::Wheres {}

        type Inner = Option<CollectionOutput<i64, T::Data>>;

        type SubOp = ();

        fn sub_op(&self, row: two_alias<<S as sqlx::Database>::Row>) -> (Self::SubOp, Self::Inner)
        where
            S: sqlx::Database,
        {
            // two_alias.try_get()
            // panic!("debug_row: {:?}", crate::debug_row::DebugRow(row.0));
            let id: Option<i64> = row.0.get(
                format!(
                    "{}{}id",
                    row.1,
                    row.2.map(|e| e.to_string()).unwrap_or_default()
                )
                .as_str(),
            );

            if let Some(id) = id {
                return (
                    (),
                    Some(CollectionOutput {
                        id: id,
                        attributes: self.to.two_alias(row).unwrap(),
                    }),
                );
            } else {
                return ((), None);
            }
        }

        type Output = Option<CollectionOutput<i64, T::Data>>;

        fn take(
            self,
            _: <Self::SubOp as Operation<S>>::Output,
            inner: Self::Inner,
        ) -> Self::Output {
            inner
        }
    }
}

mod optional_to_many_items_names {
    use core::fmt;

    use crate::{
        collections::{Collection, CollectionId},
        database_extention::DatabaseExt,
        extentions::common_expressions::StrAliased,
        from_row::{
            FromRowAlias, FromRowData, TryFromRowAlias, swich_to_base_id::pre_alias_to_base_id,
        },
        query_builder::{
            IsOpExpression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat,
        },
    };

    // "from" id exists in the sql statement, and I want attributes and id of "to"
    #[derive(Clone, Debug)]

    pub struct OptionaToManyItems<FromId, ToId, ToAttributes> {
        pub from_id: FromId,
        pub to_id: ToId,
        pub to_attributes: ToAttributes,
    }

    impl<F, Ti, Ta> StrAliased for OptionaToManyItems<F, Ti, Ta>
    where
        F: StrAliased,
        Ti: StrAliased,
        Ta: StrAliased,
    {
        type StrAliased = OptionaToManyItems<F::StrAliased, Ti::StrAliased, Ta::StrAliased>;
        fn str_aliased(&self, alias: &'static str) -> Self::StrAliased {
            OptionaToManyItems {
                from_id: self.from_id.str_aliased(alias),
                to_id: self.to_id.str_aliased(alias),
                to_attributes: self.to_attributes.str_aliased(alias),
            }
        }
    }

    impl<FromId, ToId, ToAttributes> IsOpExpression for OptionaToManyItems<FromId, ToId, ToAttributes> {
        fn is_op(&self) -> bool {
            true
        }
    }

    impl<'q, S, FromId, ToId, ToAttributes> ManyExpressions<'q, S>
        for OptionaToManyItems<FromId, ToId, ToAttributes>
    where
        S: DatabaseExt,
        FromId: ManyExpressions<'q, S>,
        ToId: ManyExpressions<'q, S>,
        ToAttributes: ManyExpressions<'q, S>,
    {
        fn expression(
            self,
            start: &'static str,
            join: &'static str,
            ctx: &mut StatementBuilder<'q, S>,
        ) where
            S: DatabaseExt,
        {
            // from is already added, I just want to add `self.to` and `self.to_attributes`
            ManyFlat((self.to_id, self.to_attributes)).expression(start, join, ctx);
        }
    }

    impl<FromId, To> FromRowData for OptionaToManyItems<FromId, To::Id, To>
    where
        FromId: CollectionId,
        To: FromRowData + Collection,
        To::Id: FromRowData,
    {
        type RData = (
            FromId::IdData,
            Option<(<To::Id as CollectionId>::IdData, To::Data)>,
        );
    }

    impl<'r, R, FromId, To> FromRowAlias<'r, R> for OptionaToManyItems<FromId, To::Id, To>
    where
        FromId: CollectionId + FromRowAlias<'r, R, RData = <FromId as CollectionId>::IdData>,
        To: Collection,
        To: FromRowAlias<'r, R, RData = <To as Collection>::Data>,
        To::Id: TryFromRowAlias<'r, R, RData = <To::Id as CollectionId>::IdData>,
        To::Data: fmt::Debug,
        FromId::IdData: fmt::Debug,
        <To::Id as CollectionId>::IdData: fmt::Debug,
        R: sqlx::Row,
        for<'q> &'q str: sqlx::ColumnIndex<R>,
        i64: sqlx::Type<R::Database> + sqlx::Decode<'r, R::Database>,
        FromId: fmt::Debug,
    {
        fn no_alias(&self, row: &'r R) -> Result<Self::RData, crate::from_row::FromRowError> {
            todo!()
        }

        fn pre_alias(
            &self,
            row: crate::from_row::RowPreAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            let found = self.to_id.try_pre_alias(row.clone())?;
            let found = if let Some(found) = found {
                Some((found, self.to_attributes.pre_alias(row.clone())?))
            } else {
                None
            };

            // panic!("debug: {:?}", self.from);

            Ok((self.from_id.pre_alias(pre_alias_to_base_id(row))?, found))
        }

        fn post_alias(
            &self,
            row: crate::from_row::RowPostAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            todo!()
        }

        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            todo!()
        }
    }
}

pub mod join_expression {
    use crate::{
        database_extention::DatabaseExt,
        query_builder::{Expression, OpExpression, StatementBuilder},
    };

    pub struct JoinExpression<ForeignTable, ForeignColumn, LocalTable, LocalColumn> {
        pub join_type: &'static str,
        pub foreign_table: ForeignTable,
        pub foreign_column: ForeignColumn,
        pub local_table: LocalTable,
        pub local_column: LocalColumn,
    }

    impl<ForeignTable, ForeignColumn, LocalTable, LocalColumn> OpExpression
        for JoinExpression<ForeignTable, ForeignColumn, LocalTable, LocalColumn>
    {
    }

    impl<'q, S, Ft, Fc, Lt, Lc> Expression<'q, S> for JoinExpression<Ft, Fc, Lt, Lc>
    where
        S: DatabaseExt,
        Ft: Expression<'q, S> + Clone,
        Fc: Expression<'q, S>,
        Lt: Expression<'q, S>,
        Lc: Expression<'q, S>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.syntax(&self.join_type);
            ctx.syntax(" ");
            self.foreign_table.clone().expression(ctx);
            ctx.syntax(" ON ");
            self.local_table.expression(ctx);
            ctx.syntax(".");
            self.local_column.expression(ctx);
            ctx.syntax(" = ");
            self.foreign_table.expression(ctx);
            ctx.syntax(".");
            self.foreign_column.expression(ctx);
        }
    }
}

mod impl_link_fetch_many {

    use crate::{
        collections::{Collection, CollectionId, SingleColumnId},
        expressions::standard_naming_conventions::ForeignKeyName,
        extentions::common_expressions::{Identifier, TableNameExpression},
        from_row::FromRowData,
        links::relation_optional_to_many::{
            OptionalToMany, join_expression::JoinExpression,
            optional_to_many_items_names::OptionaToManyItems,
        },
        operations::{CollectionOutput, OperationOutput, fetch_many::LinkFetchMany},
    };

    impl<Key, F, T> LinkFetchMany for OptionalToMany<Key, F, T>
    where
        Key: Clone + AsRef<str>,
        T: Collection<Id: SingleColumnId + Identifier> + TableNameExpression + Clone,
        F: Collection + TableNameExpression + Clone,
        // or maybe this
        OptionaToManyItems<F::Id, T::Id, T>: FromRowData<
            RData = (
                <F::Id as CollectionId>::IdData,
                Option<(<T::Id as CollectionId>::IdData, T::Data)>,
            ),
        >,
    {
        type SelectItems = OptionaToManyItems<F::Id, T::Id, T>;

        fn non_aggregating_select_items(&self) -> Self::SelectItems {
            let s = OptionaToManyItems {
                from_id: self.from.id(),
                to_id: self.to.id(),
                to_attributes: self.to.clone(),
            };

            s
        }

        type Join = JoinExpression<
            T::TableNameExpression,
            <T::Id as Identifier>::Identifier,
            F::TableNameExpression,
            ForeignKeyName<Key, T::TableNameExpression>,
        >;

        fn non_duplicating_join(&self) -> Self::Join {
            JoinExpression {
                join_type: "LEFT JOIN",
                foreign_table: self.to.table_name_expression(),
                foreign_column: self.to.id().identifier(),
                local_table: self.from.table_name_expression(),
                local_column: ForeignKeyName {
                    key: self.foriegn_key.clone(),
                    to: self.to.table_name_expression(),
                },
            }
        }

        type Wheres = ();

        fn wheres(&self) -> Self::Wheres {}

        type PostOperation = ();

        type Output = Option<CollectionOutput<<T::Id as CollectionId>::IdData, T::Data>>;

        fn take(
            &self,
            item: <Self::SelectItems as FromRowData>::RData,
            _: &mut <Self::PostOperation as OperationOutput>::Output,
        ) -> Self::Output
        where
            Self::SelectItems: FromRowData,
        {
            item.1.map(|e| CollectionOutput {
                id: e.0,
                attributes: e.1,
            })
        }

        fn post_select_each(
            &self,
            _: &<Self::SelectItems as FromRowData>::RData,
            _: &mut Self::PostOperation,
        ) where
            Self::SelectItems: FromRowData,
        {
        }

        type PostOperationInput = ();

        fn post_operation_input_init(&self) -> Self::PostOperationInput {}

        fn post_select(&self, _: Self::PostOperationInput) -> Self::PostOperation
        where
            Self::SelectItems: FromRowData,
        {
        }
    }

    // impl<Key, F, T> LinkFetchManyTakeId<F::Id> for OptionalToMany<Key, F, T>
    // where
    //     Self: LinkFetchMany<
    //         Output = Option<CollectionOutput<<T::Id as CollectionId>::IdData, T::Data>>,
    //     >,
    //     Self::PostOperation: OperationOutput,
    //     Self::SelectItems: FromRowData<
    //         RData = (
    //             <F::Id as CollectionId>::IdData,
    //             Option<(<T::Id as CollectionId>::IdData, T::Data)>,
    //         ),
    //     >,
    //     <F::Id as CollectionId>::IdData: PartialEq,
    //     F: Collection,
    //     T: Collection,
    //     <F::Id as CollectionId>::IdData: ::core::fmt::Debug + Clone,
    //     <T::Id as CollectionId>::IdData: Clone,
    //     T::Data: Clone,
    // {
    //     type ForEach = Option<CollectionOutput<<T::Id as CollectionId>::IdData, T::Data>>;
    //     type IntoIter = Vec<Option<CollectionOutput<<T::Id as CollectionId>::IdData, T::Data>>>;

    //     fn for_each(
    //         &self,
    //         into: &<F::Id as CollectionId>::IdData,
    //         item: &mut <Self::SelectItems as FromRowData>::RData,
    //         _: &mut <Self::PostOperation as OperationOutput>::Output,
    //     ) -> Self::ForEach
    //     where
    //         F::Id: CollectionId,
    //         Self::SelectItems: FromRowData,
    //         Self::PostOperation: OperationOutput,
    //     {
    //         if item.0 == *into {
    //             item.1.clone().map(|e| CollectionOutput {
    //                 id: e.0,
    //                 attributes: e.1,
    //             })
    //         } else {
    //             None
    //         }
    //     }

    //     fn take(
    //         &self,
    //         mut all: Self::IntoIter,
    //         _: &<F::Id as CollectionId>::IdData,
    //         _: &mut <Self::PostOperation as OperationOutput>::Output,
    //     ) -> Self::Output
    //     where
    //         Self::SelectItems: FromRowData,
    //         Self::PostOperation: OperationOutput,
    //         F::Id: CollectionId,
    //     {
    //         if all.len() > 1 {
    //             panic!("bug: optional to many should have at maximum one item");
    //         }
    //         all.pop().flatten()
    //     }
    // }
}
