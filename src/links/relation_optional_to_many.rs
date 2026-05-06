use crate::links::{LinkedToBase, LinkedViaId};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct OptionalToMany<Id, F, T> {
    pub fk_unique_id: Id,
    pub from: F,
    pub to: T,
}

pub mod fk_name {
    use core::fmt;

    use crate::{
        collections::Collection,
        database_extention::DatabaseExt,
        extentions::common_expressions::TableNameExpression,
        links::relation_optional_to_many::OptionalToMany,
        query_builder::{Expression, OpExpression, StatementBuilder},
    };

    pub struct AsIdentifier<Relation> {
        pub relation: Relation,
    }

    impl<Id, F, T> OptionalToMany<Id, F, T>
    where
        Id: Clone,
        F: Clone,
        T: Clone,
    {
        pub fn fk_name(&self) -> AsIdentifier<Self> {
            AsIdentifier {
                relation: self.clone(),
            }
        }
    }

    impl<Id, F, T> fmt::Display for AsIdentifier<OptionalToMany<Id, F, T>>
    where
        Id: AsRef<str>,
        T: Collection,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "fk_{}{}",
                self.relation.to.table_name_lower_case(),
                self.relation.fk_unique_id.as_ref(),
            )
        }
    }

    impl<Id, F, T> OpExpression for AsIdentifier<OptionalToMany<Id, F, T>> {}

    impl<'q, S, Id, F, T> Expression<'q, S> for AsIdentifier<OptionalToMany<Id, F, T>>
    where
        S: DatabaseExt,
        Id: 'q + AsRef<str>,
        F: 'q,
        T: 'q + TableNameExpression<LowerCaseTableNameExpression: AsRef<str>>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.sanitize_strings((
                "fk_",
                self.relation.to.lower_case_table_name_expression().as_ref(),
                self.relation.fk_unique_id.as_ref(),
            ));
        }
    }
}

pub mod find_place_for_this {
    use std::marker::PhantomData;

    use sqlx::{ColumnIndex, Decode, Row, Type};

    use crate::from_row::{
        FromRowAlias, FromRowData, FromRowError, RowPostAliased, RowPreAliased, RowTwoAliased,
    };

    pub struct OneColumn<Name, ExpectedType> {
        pub as_name: Name,
        pub as_type: PhantomData<ExpectedType>,
    }

    impl<AsName, AsType> FromRowData for OneColumn<AsName, AsType> {
        type RData = AsType;
    }

    impl<'r, R, AsName, AsType> FromRowAlias<'r, R> for OneColumn<AsName, AsType>
    where
        AsName: ToString,
        AsType: for<'q> Decode<'q, R::Database> + Type<R::Database>,
        R: Row,
        for<'q> &'q str: ColumnIndex<R>,
    {
        fn no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError> {
            let data: AsType = row.try_get(self.as_name.to_string().as_str())?;
            Ok(data)
        }
        fn pre_alias(&self, row: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            let data: AsType = row.try_get(self.as_name.to_string().as_str())?;
            Ok(data)
        }
        fn post_alias(&self, _: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            panic!("debug in the process of deprecating this method");
        }
        fn two_alias(&self, row: RowTwoAliased<'r, R>) -> Result<Self::RData, FromRowError> {
            let data: AsType = row.try_get(self.as_name.to_string().as_str())?;
            Ok(data)
        }
    }
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
                        key: self.fk_unique_id.clone(),
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
        extentions::common_expressions::Aliased,
        from_row::{
            FromRowAlias, FromRowData, TryFromRowAlias,
            swich_to_base_id::{pre_alias_to_base_id, two_alias_to_base_id},
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

    impl<F, Ti, Ta> Aliased for OptionaToManyItems<F, Ti, Ta>
    where
        F: Aliased,
        Ti: Aliased,
        Ta: Aliased,
    {
        type Aliased = OptionaToManyItems<F::Aliased, Ti::Aliased, Ta::Aliased>;
        fn aliased(&self, alias: &'static str) -> Self::Aliased {
            OptionaToManyItems {
                from_id: self.from_id.aliased(alias),
                to_id: self.to_id.aliased(alias),
                to_attributes: self.to_attributes.aliased(alias),
            }
        }
        type NumAliased = OptionaToManyItems<F::NumAliased, Ti::NumAliased, Ta::NumAliased>;
        fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
            OptionaToManyItems {
                from_id: self.from_id.num_aliased(num, alias),
                to_id: self.to_id.num_aliased(num, alias),
                to_attributes: self.to_attributes.num_aliased(num, alias),
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
            let _ = row;
            todo!()
        }

        fn pre_alias(
            &self,
            row: crate::from_row::RowPreAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            let try_to_find_id = self.to_id.try_pre_alias(row.clone())?;
            let found = if let Some(found) = try_to_find_id {
                Some((found, self.to_attributes.pre_alias(row.clone())?))
            } else {
                None
            };

            Ok((self.from_id.pre_alias(pre_alias_to_base_id(row))?, found))
        }

        fn post_alias(
            &self,
            row: crate::from_row::RowPostAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            let _ = row;
            panic!("debug in the process of deprecating this method");
        }

        fn two_alias(
            &self,
            row: crate::from_row::RowTwoAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            let try_to_find_id = self.to_id.try_two_alias(row.clone())?;
            let found = if let Some(found) = try_to_find_id {
                Some((found, self.to_attributes.two_alias(row.clone())?))
            } else {
                None
            };

            Ok((self.from_id.two_alias(two_alias_to_base_id(row))?, found))
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
        operations::{CollectionOutput, OperationOutput, fetch_many::LinkFetch},
    };

    impl<Key, F, T> LinkFetch for OptionalToMany<Key, F, T>
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

        fn non_duplicating_join_expressions(&self) -> Self::Join {
            JoinExpression {
                join_type: "LEFT JOIN",
                foreign_table: self.to.table_name_expression(),
                foreign_column: self.to.id().identifier(),
                local_table: self.from.table_name_expression(),
                local_column: ForeignKeyName {
                    key: self.fk_unique_id.clone(),
                    to: self.to.table_name_expression(),
                },
            }
        }

        type Wheres = ();

        fn where_expressions(&self) -> Self::Wheres {}

        type Op = ();

        type Output = Option<CollectionOutput<<T::Id as CollectionId>::IdData, T::Data>>;

        fn take_many(
            &self,
            item: <Self::SelectItems as FromRowData>::RData,
            _: &mut <Self::Op as OperationOutput>::Output,
        ) -> Self::Output
        where
            Self::SelectItems: FromRowData,
        {
            item.1.map(|e| CollectionOutput {
                id: e.0,
                attributes: e.1,
            })
        }

        fn operation_fix_on_many(
            &self,
            _: &<Self::SelectItems as FromRowData>::RData,
            _: &mut Self::Op,
        ) where
            Self::SelectItems: FromRowData,
        {
        }

        type OpInput = ();

        fn operation_initialize_input(&self) -> Self::OpInput {}

        fn operation_construct(&self, _: Self::OpInput) -> Self::Op
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

mod impl_set_new_for_insert {
    use std::marker::PhantomData;

    use crate::{
        collections::{AutoGenerate, Collection, CollectionId},
        links::{
            relation_optional_to_many::{OptionalToMany, fk_name::AsIdentifier},
            set_new_mod::SetNew,
        },
        operations::{
            CollectionOutput, OperationOutput,
            insert_one::{
                ConstraintViolation, InsertLinkConsumeData, InsertLinkData, InsertOne,
                InsertOneLink,
            },
        },
        query_builder::Bind,
    };

    impl<Key, From, To> InsertLinkConsumeData for SetNew<OptionalToMany<Key, From, To>, To::Data>
    where
        To: Collection,
        Key: Clone,
        From: Clone,
        To: Clone,
        <To::Id as CollectionId>::IdData: Clone,
    {
        type Link = SetNew<OptionalToMany<Key, From, To>, PhantomData<To::Data>>;

        fn consume_data(
            self,
        ) -> (
            Self::Link,
            crate::operations::insert_one::InsertLinkData<
                <Self::Link as crate::operations::insert_one::InsertOneLink>::PreOpData,
                <Self::Link as crate::operations::insert_one::InsertOneLink>::InsertValuesData,
                <Self::Link as crate::operations::insert_one::InsertOneLink>::PostOpData,
            >,
        ) {
            (
                SetNew {
                    relation: self.relation,
                    data: PhantomData,
                },
                InsertLinkData {
                    pre_op_data: self.data,
                    insert_value_data: (),
                    post_op_data: (),
                },
            )
        }
    }

    impl<Key, From, To> InsertOneLink for SetNew<OptionalToMany<Key, From, To>, PhantomData<To::Data>>
    where
        To: Clone,
        From: Clone,
        Key: Clone,
        To: Collection,
        To::Id: CollectionId<IdData: Clone>,
    {
        type PreOp = InsertOne<AutoGenerate, To, To::Data, ()>;

        type PreOpData = To::Data;

        fn pre_operation_init(&self, input: Self::PreOpData) -> Self::PreOp {
            InsertOne {
                id: AutoGenerate,
                data: input,
                base: self.relation.to.clone(),
                links: (),
            }
        }

        type PreOpError = ConstraintViolation;

        fn pre_op_split(
            &self,
            pre_op_output: <Self::PreOp as OperationOutput>::Output,
        ) -> Result<
            (
                Self::PreOpToInsertValue,
                Self::PreOpToTake,
                Self::PreOpToPostOp,
            ),
            Self::PreOpError,
        > {
            let unwrapped = pre_op_output?;
            Ok((unwrapped.id.clone(), unwrapped.into(), ()))
        }

        type PostOpError = ();

        fn post_op_error(
            &self,
            _: &<Self::PostOp as OperationOutput>::Output,
        ) -> Result<(), Self::PostOpError> {
            Ok(())
        }

        type PreOpToInsertValue = <To::Id as CollectionId>::IdData;
        type PreOpToTake = CollectionOutput<<To::Id as CollectionId>::IdData, To::Data>;
        type PreOpToPostOp = ();

        type InsertNames = AsIdentifier<OptionalToMany<Key, From, To>>;

        fn insert_names(&self) -> Self::InsertNames {
            self.relation.fk_name()
        }

        type InsertReturning = AsIdentifier<OptionalToMany<Key, From, To>>;

        fn insert_returning(&self) -> Self::InsertReturning {
            self.relation.fk_name()
        }

        type InsertValuesData = ();

        type InsertValues = Bind<<To::Id as CollectionId>::IdData>;

        fn insert_value(&self, _: (), id: Self::PreOpToInsertValue) -> Self::InsertValues {
            Bind(id)
        }

        type FromRow = ();

        fn from_row(&self) -> Self::FromRow {}

        type TakeInput = ();

        type PostOp = ();

        type PostOpData = ();

        fn from_row_result(
            &self,
            _: Self::PostOpData,
            _: <Self::FromRow as crate::from_row::FromRowData>::RData,
            _: Self::PreOpToPostOp,
        ) -> (Self::PostOp, Self::TakeInput) {
            ((), ())
        }

        type Output = CollectionOutput<<To::Id as CollectionId>::IdData, To::Data>;

        fn take(
            self,
            _: <Self::PostOp as crate::operations::OperationOutput>::Output,
            _: Self::TakeInput,
            take: Self::PreOpToTake,
        ) -> Self::Output {
            take
        }
    }

    #[cfg(test)]
    mod test {
        use sqlx::Sqlite;

        use crate::{
            collections::AutoGenerate,
            connect_in_memory::ConnectInMemory,
            links::{Link, set_new_mod::SetNew},
            operations::{CollectionOutput, LinkedOutput, Operation, insert_one::InsertOne},
            test_module::{self, Category, Todo, category},
        };

        #[tokio::test]
        async fn test_insert_one_set_new() {
            let mut conn = Sqlite::connect_in_memory_2().await;

            sqlx::query(
                "
                CREATE TABLE Category (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL
                );
                CREATE TABLE Todo (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    done BOOLEAN NOT NULL,
                    description TEXT,
                    fk_category_def INTEGER,
                    FOREIGN KEY (fk_category_def) REFERENCES Category(id)
                );
                ",
            )
            .execute(&mut conn)
            .await
            .unwrap();

            let output = Operation::<Sqlite>::exec_operation(
                InsertOne {
                    id: AutoGenerate,
                    base: test_module::todo,
                    data: Todo {
                        title: "first_todo".to_string(),
                        done: true,
                        description: None,
                    },
                    links: SetNew {
                        data: Category {
                            title: "category_1".to_string(),
                        },
                        relation: <category as Link<test_module::todo>>::spec(category),
                    },
                },
                &mut conn,
            )
            .await;

            pretty_assertions::assert_eq!(
                output,
                Ok(LinkedOutput {
                    id: 1,
                    attributes: Todo {
                        title: "first_todo".to_string(),
                        done: true,
                        description: None,
                    },
                    links: CollectionOutput {
                        id: 1,
                        attributes: Category {
                            title: "category_1".to_string(),
                        },
                    },
                })
            );
        }
    }
}

mod impl_set_id_for_insert {
    use std::marker::PhantomData;

    use crate::{
        collections::{Collection, CollectionId},
        expressions::ColumnEqual,
        extentions::common_expressions::Identifier,
        links::{
            relation_optional_to_many::{
                OptionalToMany, find_place_for_this::OneColumn, fk_name::AsIdentifier,
            },
            set_id_mod::SetId,
        },
        operations::{
            CollectionOutput, LinkedOutput, OperationOutput,
            fetch_one::FetchOne,
            insert_one::{InsertLinkConsumeData, InsertLinkData, InsertOneLink},
        },
        query_builder::Bind,
    };

    impl<Key, From, To> InsertLinkConsumeData
        for SetId<OptionalToMany<Key, From, To>, <To::Id as CollectionId>::IdData>
    where
        To: Collection,
        Key: Clone,
        From: Clone,
        To: Clone,
        To::Id: Identifier,
    {
        type Link =
            SetId<OptionalToMany<Key, From, To>, PhantomData<<To::Id as CollectionId>::IdData>>;

        fn consume_data(
            self,
        ) -> (
            Self::Link,
            crate::operations::insert_one::InsertLinkData<
                <Self::Link as InsertOneLink>::PreOpData,
                <Self::Link as InsertOneLink>::InsertValuesData,
                <Self::Link as InsertOneLink>::PostOpData,
            >,
        ) {
            (
                SetId {
                    relation: self.relation,
                    id: PhantomData,
                },
                InsertLinkData {
                    pre_op_data: (),
                    insert_value_data: self.id,
                    post_op_data: (),
                },
            )
        }
    }

    pub struct ForeignKeyFor<Relation> {
        pub relation: Relation,
    }

    impl<Key, From, To> OptionalToMany<Key, From, To>
    where
        Self: Clone,
    {
        pub fn foreign_key(&self) -> ForeignKeyFor<Self> {
            ForeignKeyFor {
                relation: self.clone(),
            }
        }
    }

    impl<Key, From, To> InsertOneLink
        for SetId<OptionalToMany<Key, From, To>, PhantomData<<To::Id as CollectionId>::IdData>>
    where
        To: Collection,
        Key: Clone,
        From: Clone,
        To: Clone,
        To::Id: Identifier,
    {
        type PreOp = ();
        type PreOpError = ();

        type PreOpData = ();

        fn pre_operation_init(&self, _: Self::PreOpData) -> Self::PreOp {}

        fn pre_op_split(
            &self,
            _: <Self::PreOp as OperationOutput>::Output,
        ) -> Result<
            (
                Self::PreOpToInsertValue,
                Self::PreOpToTake,
                Self::PreOpToPostOp,
            ),
            Self::PreOpError,
        > {
            Ok(((), (), ()))
        }

        type PreOpToInsertValue = ();
        type PreOpToTake = ();
        type PreOpToPostOp = ();

        type InsertNames = AsIdentifier<OptionalToMany<Key, From, To>>;

        fn insert_names(&self) -> Self::InsertNames {
            self.relation.fk_name()
        }

        type InsertReturning = AsIdentifier<OptionalToMany<Key, From, To>>;

        fn insert_returning(&self) -> Self::InsertReturning {
            self.relation.fk_name()
        }

        type InsertValuesData = <To::Id as CollectionId>::IdData;

        type InsertValues = Bind<<To::Id as CollectionId>::IdData>;

        fn insert_value(&self, from_data: Self::InsertValuesData, _: ()) -> Self::InsertValues {
            Bind(from_data)
        }

        type FromRow = OneColumn<
            AsIdentifier<OptionalToMany<Key, From, To>>,
            <To::Id as CollectionId>::IdData,
        >;

        fn from_row(&self) -> Self::FromRow {
            OneColumn {
                as_name: self.relation.fk_name(),
                as_type: PhantomData,
            }
        }

        type TakeInput = ();

        type PostOp = FetchOne<
            To,
            (),
            ColumnEqual<<To::Id as Identifier>::Identifier, <To::Id as CollectionId>::IdData>,
        >;

        type PostOpError = ();
        fn post_op_error(
            &self,
            _: &<Self::PostOp as OperationOutput>::Output,
        ) -> Result<(), Self::PostOpError> {
            Ok(())
        }

        type PostOpData = ();

        fn from_row_result(
            &self,
            _: (),
            from_row: <To::Id as CollectionId>::IdData,
            _: Self::PreOpToPostOp,
        ) -> (Self::PostOp, Self::TakeInput) {
            (
                FetchOne {
                    base: self.relation.to.clone(),
                    links: (),
                    wheres: ColumnEqual {
                        col: self.relation.to.id().identifier(),
                        eq: from_row,
                    },
                },
                (),
            )
        }

        type Output = CollectionOutput<<To::Id as CollectionId>::IdData, To::Data>;

        fn take(
            self,
            pre_op: Option<LinkedOutput<<To::Id as CollectionId>::IdData, To::Data, ()>>,
            _: (),
            _: Self::PreOpToTake,
        ) -> Self::Output {
            pre_op.expect("sql query should have failed by now").into()
        }
    }

    #[cfg(test)]
    mod test {
        use sqlx::Sqlite;

        use crate::{
            collections::AutoGenerate,
            connect_in_memory::ConnectInMemory,
            links::{Link, set_id_mod::SetId},
            operations::{CollectionOutput, LinkedOutput, Operation, insert_one::InsertOne},
            test_module::{self, Category, Todo, category},
        };

        #[tokio::test]
        async fn test_insert_one() {
            let mut conn = Sqlite::connect_in_memory_2().await;

            sqlx::query(
                "
                CREATE TABLE Category (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL
                );
                CREATE TABLE Todo (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    done BOOLEAN NOT NULL,
                    description TEXT,
                    fk_category_def INTEGER,
                    FOREIGN KEY (fk_category_def) REFERENCES Category(id)
                );

                INSERT INTO Category (title) VALUES ('category_1');
                ",
            )
            .execute(&mut conn)
            .await
            .unwrap();

            let output = Operation::<Sqlite>::exec_operation(
                InsertOne {
                    id: AutoGenerate,
                    base: test_module::todo,
                    data: Todo {
                        title: "first_todo".to_string(),
                        done: true,
                        description: None,
                    },
                    links: SetId {
                        id: 1,
                        relation: <category as Link<test_module::todo>>::spec(category),
                    },
                },
                &mut conn,
            )
            .await;

            pretty_assertions::assert_eq!(
                output,
                Ok(LinkedOutput {
                    id: 1,
                    attributes: Todo {
                        title: "first_todo".to_string(),
                        done: true,
                        description: None,
                    },
                    links: CollectionOutput {
                        id: 1,
                        attributes: Category {
                            title: "category_1".to_string(),
                        },
                    },
                })
            );
        }
    }
}
mod impl_set_id_for_insert_v0 {
    use crate::{
        collections::{Collection, CollectionId},
        database_extention::DatabaseExt,
        extentions::common_expressions::{Identifier, TableNameExpression, V0OnInsert},
        from_row::{FromRowAlias, FromRowData},
        links::{relation_optional_to_many::OptionalToMany, set_id_mod::SetId},
        operations::v1_insert_one::InsertLink,
        query_builder::{Bind, Expression, OpExpression, StatementBuilder},
    };

    #[derive(Clone)]
    pub struct InsertItem<ToTableName, Key, ToId> {
        pub to_table_name: ToTableName,
        pub key: Key,
        pub to_id: ToId,
    }

    pub struct LocalForeignKeyIdent<ToTableName, Key> {
        pub to_table_name: ToTableName,
        pub key: Key,
    }

    impl<ToTableName: Clone, Key: Clone, ToId> Identifier for InsertItem<ToTableName, Key, ToId> {
        type Identifier = LocalForeignKeyIdent<ToTableName, Key>;

        fn identifier(&self) -> Self::Identifier {
            LocalForeignKeyIdent {
                to_table_name: self.to_table_name.clone(),
                key: self.key.clone(),
            }
        }
    }

    impl<ToTableName, Key> OpExpression for LocalForeignKeyIdent<ToTableName, Key> {}
    impl<'q, S: DatabaseExt, ToTableName: 'q + AsRef<str>, Key: 'q + AsRef<str>> Expression<'q, S>
        for LocalForeignKeyIdent<ToTableName, Key>
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.sanitize_strings(("fk_", self.to_table_name.as_ref(), self.key.as_ref()));
        }
    }

    impl<ToTableName, Key, ToId> V0OnInsert for InsertItem<ToTableName, Key, ToId> {
        type InsertInput = ();
        type InsertExpression = Bind<ToId>;

        fn on_insert(self, _: Self::InsertInput) -> Self::InsertExpression {
            Bind(self.to_id)
        }
    }

    impl<ToTableName, Key, SetId> FromRowData for InsertItem<ToTableName, Key, SetId> {
        type RData = ();
    }

    impl<'r, R, ToTableName, Key, SetId> FromRowAlias<'r, R> for InsertItem<ToTableName, Key, SetId> {
        fn no_alias(&self, _: &'r R) -> Result<Self::RData, crate::from_row::FromRowError> {
            Ok(())
        }

        fn pre_alias(
            &self,
            _: crate::from_row::RowPreAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            Ok(())
        }

        fn post_alias(
            &self,
            _: crate::from_row::RowPostAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            Ok(())
        }

        fn two_alias(
            &self,
            _: crate::from_row::RowTwoAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            Ok(())
        }
    }

    impl<Key, From, To> InsertLink
        for SetId<OptionalToMany<Key, From, To>, <To::Id as CollectionId>::IdData>
    where
        To: Collection,
        To: TableNameExpression,
        <To::Id as CollectionId>::IdData: Clone,
        Key: Clone,
    {
        type PreOp = ();

        fn pre_operation(&self) -> Self::PreOp {}

        type InsertItems =
            InsertItem<To::LowerCaseTableNameExpression, Key, <To::Id as CollectionId>::IdData>;

        fn insert_items(&self, _: ()) -> Self::InsertItems
        where
            Self::PreOp: crate::operations::OperationOutput,
        {
            InsertItem {
                to_table_name: self.relation.to.lower_case_table_name_expression(),
                to_id: self.id.clone(),
                key: self.relation.fk_unique_id.clone(),
            }
        }

        type PostOp = ();

        fn post_operation(
            &self,
            _: &<Self::InsertItems as crate::from_row::FromRowData>::RData,
        ) -> Self::PostOp
        where
            Self::InsertItems: crate::from_row::FromRowData,
        {
        }

        type Output = <To::Id as CollectionId>::IdData;

        fn take(
            self,
            _: <Self::PostOp as crate::operations::OperationOutput>::Output,
            _: <Self::InsertItems as crate::from_row::FromRowData>::RData,
        ) -> Self::Output
        where
            Self::PostOp: crate::operations::OperationOutput,
            Self::InsertItems: crate::from_row::FromRowData,
        {
            self.id
        }
    }
}
