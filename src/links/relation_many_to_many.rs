use crate::links::{LinkedToBase, LinkedViaIds};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ManyToMany<Key, From, To> {
    pub relation_key: Key,
    pub from: From,
    pub to: To,
}

impl<Key, From, To> LinkedViaIds for ManyToMany<Key, From, To> {}

impl<Key, From, To> LinkedToBase for ManyToMany<Key, From, To> {
    type Base = From;
}

pub mod junction_names {
    use core::fmt;

    use crate::{
        database_extention::DatabaseExt,
        expressions::standard_naming_conventions::ConjuctionTableName,
        extentions::common_expressions::TableNameExpression,
        links::relation_many_to_many::ManyToMany,
        sqlx_query_builder::{Expression, OpExpression, StatementBuilder},
    };

    #[derive(Clone)]
    pub struct JunctionSideColumn<TableLower> {
        pub table_lower: TableLower,
    }

    impl<TableLower> fmt::Display for JunctionSideColumn<TableLower>
    where
        TableLower: AsRef<str>,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}_id", self.table_lower.as_ref())
        }
    }

    impl<TableLower> OpExpression for JunctionSideColumn<TableLower> {}

    impl<'q, S, TableLower> Expression<'q, S> for JunctionSideColumn<TableLower>
    where
        S: DatabaseExt,
        TableLower: AsRef<str> + 'q,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.sanitize_many((self.table_lower.as_ref(), "_id"));
        }
    }

    impl<Key, From, To> ManyToMany<Key, From, To>
    where
        Key: Clone,
        From: Clone + TableNameExpression,
        To: Clone + TableNameExpression,
    {
        pub fn junction_table_name(
            &self,
        ) -> ConjuctionTableName<
            From::LowerCaseTableNameExpression,
            To::LowerCaseTableNameExpression,
            Key,
        >
        where
            Key: Clone,
        {
            ConjuctionTableName {
                first: self.from.lower_case_table_name_expression(),
                second: self.to.lower_case_table_name_expression(),
                key: self.relation_key.clone(),
            }
        }

        pub fn from_junction_column(
            &self,
        ) -> JunctionSideColumn<From::LowerCaseTableNameExpression> {
            JunctionSideColumn {
                table_lower: self.from.lower_case_table_name_expression(),
            }
        }

        pub fn to_junction_column(&self) -> JunctionSideColumn<To::LowerCaseTableNameExpression> {
            JunctionSideColumn {
                table_lower: self.to.lower_case_table_name_expression(),
            }
        }
    }
}

mod migration_expressions {
    use crate::{
        database_extention::DatabaseExt,
        sqlx_query_builder::{Expression, ManyExpressions, OpExpression, StatementBuilder},
    };

    pub struct OnDeleteCascade;

    impl OpExpression for OnDeleteCascade {}

    impl<'q, S> Expression<'q, S> for OnDeleteCascade
    where
        S: DatabaseExt,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.syntax("ON DELETE CASCADE");
        }
    }

    pub struct CompositePrimaryKey<Cols>(pub Cols);

    impl<Cols> OpExpression for CompositePrimaryKey<Cols> {}

    impl<'q, S, Cols> Expression<'q, S> for CompositePrimaryKey<Cols>
    where
        S: DatabaseExt,
        Cols: ManyExpressions<'q, S> + 'q,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.syntax("PRIMARY KEY (");
            self.0.expression("", ", ", ctx);
            ctx.syntax(")");
        }
    }
}

mod impl_on_migrate {
    use std::marker::PhantomData;

    use crate::{
        collections::{Collection, SingleColumnId},
        extentions::common_expressions::TableNameExpression,
        links::relation_many_to_many::{
            ManyToMany,
            junction_names::JunctionSideColumn,
            migration_expressions::{CompositePrimaryKey, OnDeleteCascade},
        },
        on_migrate::OnMigrate,
        sqlx_query_builder::basic_expressions::{ColumnDefinition, ManyFlat, foriegn_key},
        sqlx_query_builder::statements::create_table_statement::{
            CreateTable, expressions::create_table,
        },
    };

    impl<Key, From, To> OnMigrate for ManyToMany<Key, From, To>
    where
        Key: AsRef<str> + Clone,
        From: Collection<Id: SingleColumnId> + Clone + TableNameExpression,
        To: Collection<Id: SingleColumnId> + Clone + TableNameExpression,
    {
        type Statements = CreateTable<
            create_table,
            ConjuctionTableName<
                From::LowerCaseTableNameExpression,
                To::LowerCaseTableNameExpression,
                Key,
            >,
            ManyPossible<(
                ColumnDefinition<
                    JunctionSideColumn<From::LowerCaseTableNameExpression>,
                    i64,
                    foriegn_key<ManyPossible<(OnDeleteCascade,)>>,
                >,
                ColumnDefinition<
                    JunctionSideColumn<To::LowerCaseTableNameExpression>,
                    i64,
                    foriegn_key<ManyPossible<(OnDeleteCascade,)>>,
                >,
                CompositePrimaryKey<
                    ManyFlat<(
                        JunctionSideColumn<From::LowerCaseTableNameExpression>,
                        JunctionSideColumn<To::LowerCaseTableNameExpression>,
                    )>,
                >,
            )>,
        >;

        fn statments(&self) -> Self::Statements {
            CreateTable {
                init: create_table,
                name: self.junction_table_name(),
                col_defs: ManyPossible((
                    ColumnDefinition {
                        name: self.from_junction_column(),
                        ty: PhantomData::<i64>,
                        constraints: foriegn_key {
                            references_table: self.from.table_name().to_string(),
                            references_col: self.from.id().as_ref().to_string(),
                            ons: ManyPossible((OnDeleteCascade,)),
                        },
                    },
                    ColumnDefinition {
                        name: self.to_junction_column(),
                        ty: PhantomData::<i64>,
                        constraints: foriegn_key {
                            references_table: self.to.table_name().to_string(),
                            references_col: self.to.id().as_ref().to_string(),
                            ons: ManyPossible((OnDeleteCascade,)),
                        },
                    },
                    CompositePrimaryKey(ManyFlat((
                        self.from_junction_column(),
                        self.to_junction_column(),
                    ))),
                )),
            }
        }
    }
}

mod many_to_many_items {
    use crate::{
        collections::{Collection, CollectionId},
        database_extention::DatabaseExt,
        extentions::common_expressions::Aliased,
        from_row::{
            FromRowAlias, FromRowData, TryFromRowAlias,
            swich_to_base_id::{pre_alias_to_base_id, two_alias_to_base_id},
        },
        sqlx_query_builder::{
            IsOpExpression, ManyExpressions, StatementBuilder, functional_expr::ManyFlat,
        },
    };

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    pub struct ManyToManyItems<FromId, ToId, ToAttributes> {
        pub from_id: FromId,
        pub to_id: ToId,
        pub to_attributes: ToAttributes,
    }

    impl<F, Ti, Ta> Aliased for ManyToManyItems<F, Ti, Ta>
    where
        F: Aliased,
        Ti: Aliased,
        Ta: Aliased,
    {
        type Aliased = ManyToManyItems<F::Aliased, Ti::Aliased, Ta::Aliased>;
        fn aliased(&self, alias: &'static str) -> Self::Aliased {
            ManyToManyItems {
                from_id: self.from_id.aliased(alias),
                to_id: self.to_id.aliased(alias),
                to_attributes: self.to_attributes.aliased(alias),
            }
        }
        type NumAliased = ManyToManyItems<F::NumAliased, Ti::NumAliased, Ta::NumAliased>;
        fn num_aliased(&self, num: usize, alias: &'static str) -> Self::NumAliased {
            ManyToManyItems {
                from_id: self.from_id.num_aliased(num, alias),
                to_id: self.to_id.num_aliased(num, alias),
                to_attributes: self.to_attributes.num_aliased(num, alias),
            }
        }
    }

    impl<FromId, ToId, ToAttributes> IsOpExpression for ManyToManyItems<FromId, ToId, ToAttributes> {
        fn is_op(&self) -> bool {
            true
        }
    }

    impl<'q, S, FromId, ToId, ToAttributes> ManyExpressions<'q, S>
        for ManyToManyItems<FromId, ToId, ToAttributes>
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
        ) {
            ManyFlat((self.to_id, self.to_attributes)).expression(start, join, ctx);
        }
    }

    impl<FromId, To> FromRowData for ManyToManyItems<FromId, To::Id, To>
    where
        FromId: CollectionId,
        To: FromRowData + Collection,
        To::Id: FromRowData,
    {
        type RData = (
            FromId::IdData,
            Option<(<To::Id as CollectionId>::IdData, To::OutputData)>,
        );
    }

    impl<'r, R, FromId, To> FromRowAlias<'r, R> for ManyToManyItems<FromId, To::Id, To>
    where
        FromId: CollectionId + FromRowAlias<'r, R, RData = <FromId as CollectionId>::IdData>,
        To: Collection,
        To: FromRowAlias<'r, R, RData = <To as Collection>::OutputData>,
        To::Id: TryFromRowAlias<'r, R, RData = <To::Id as CollectionId>::IdData>,
        R: sqlx::Row,
        for<'q> &'q str: sqlx::ColumnIndex<R>,
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
            _: crate::from_row::RowPostAliased<'r, R>,
        ) -> Result<Self::RData, crate::from_row::FromRowError>
        where
            R: sqlx::Row,
        {
            panic!("to be deprecated")
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

mod many_to_many_joins {
    use crate::{
        database_extention::DatabaseExt,
        sqlx_query_builder::{Expression, ManyExpressions, OpExpression, StatementBuilder},
    };

    #[allow(dead_code)]
    pub struct ManyToManyJoins<JunctionJoin, ToJoin>(pub JunctionJoin, pub ToJoin);

    impl<JunctionJoin, ToJoin> OpExpression for ManyToManyJoins<JunctionJoin, ToJoin> {}

    impl<'q, S, JunctionJoin, ToJoin> Expression<'q, S> for ManyToManyJoins<JunctionJoin, ToJoin>
    where
        S: DatabaseExt,
        JunctionJoin: ManyExpressions<'q, S> + 'q,
        ToJoin: ManyExpressions<'q, S> + 'q,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            self.0.expression(" ", " ", ctx);
            if self.1.is_op() {
                self.1.expression(" ", " ", ctx);
            }
        }
    }
}

mod impl_link_fetch_many {
    use std::collections::HashSet;

    use crate::{
        collections::{Collection, CollectionId, SingleColumnId},
        extentions::{
            Members,
            common_expressions::{Aliased, TableNameExpression},
        },
        from_row::FromRowData,
        links::relation_many_to_many::ManyToMany,
        operations::{
            CollectionOutput, ManyLinkOutput, OperationOutput,
            fetch_linked_records::{FetchManyToManyLinked, ManyToManyLinkedMap},
            fetch_many::LinkFetch,
        },
    };

    impl<Key, From, To> LinkFetch for ManyToMany<Key, From, To>
    where
        Key: Clone + AsRef<str>,
        From: Collection<Id: SingleColumnId + Aliased> + TableNameExpression + Clone,
        To: Collection<Id: SingleColumnId> + TableNameExpression + Members + Clone,
        <From::Id as CollectionId>::IdData: Copy + Clone + std::hash::Hash + Eq,
        From::Id: FromRowData<RData = <From::Id as CollectionId>::IdData>,
        FetchManyToManyLinked<Key, From, To>: OperationOutput<
            Output = ManyToManyLinkedMap<
                <From::Id as CollectionId>::IdData,
                <To::Id as CollectionId>::IdData,
                To::OutputData,
            >,
        >,
    {
        type SelectItems = From::Id;

        fn non_aggregating_select_items(&self) -> Self::SelectItems {
            self.from.id()
        }

        type Join = ();

        fn non_duplicating_join_expressions(&self) -> Self::Join {}

        type Wheres = ();

        fn where_expressions(&self) -> Self::Wheres {}

        type Op = FetchManyToManyLinked<Key, From, To>;

        type Output =
            ManyLinkOutput<CollectionOutput<<To::Id as CollectionId>::IdData, To::OutputData>>;

        fn take_many(
            &self,
            from_id: <Self::SelectItems as FromRowData>::RData,
            op: &mut <Self::Op as OperationOutput>::Output,
        ) -> Self::Output
        where
            Self::SelectItems: FromRowData,
        {
            ManyLinkOutput {
                many_output: op.remove(&from_id).unwrap_or_default(),
            }
        }

        type OpInput = Vec<<From::Id as CollectionId>::IdData>;

        fn operation_initialize_input(&self) -> Self::OpInput {
            Vec::new()
        }

        fn operation_fix_on_many(
            &self,
            from_id: &<Self::SelectItems as FromRowData>::RData,
            input: &mut Self::OpInput,
        ) where
            Self::SelectItems: FromRowData,
        {
            input.push(*from_id);
        }

        fn operation_construct(&self, input: Self::OpInput) -> Self::Op
        where
            Self::SelectItems: FromRowData,
        {
            let mut seen = HashSet::new();
            let from_ids = input.into_iter().filter(|id| seen.insert(*id)).collect();
            FetchManyToManyLinked::new(self.clone(), from_ids)
        }
    }
}

pub type ManyToManyFetchOne<Key, From, To> = ManyToMany<Key, From, To>;

mod impl_mutate_links {
    use std::marker::PhantomData;

    use crate::{
        collections::{Collection, CollectionId, SingleColumnId},
        expressions::{ColumnEqual, single_col_expressions::UpdatingCol},
        extentions::{
            Members,
            common_expressions::{Identifier, Scoped, TableNameExpression},
        },
        from_row::FromRowData,
        links::{
            relation_many_to_many::ManyToMany,
            relation_optional_to_many::find_place_for_this::OneColumn, update_links::SetId,
        },
        operations::{
            CollectionOutput, LinkedOutput, ManyLinkOutput, OperationOutput,
            delete::{DeleteLink, DeleteLinkData, DeleteLinkPreOp, DeleteLinkSplit},
            fetch_linked_records::{FetchManyToManyLinked, ManyToManyLinkedMap},
            fetch_one::FetchOne,
            insert_one::{
                ConstraintViolation, InsertLinkConsumeData, InsertLinkData, InsertOneLink,
            },
            junction::{DeleteJunctionRow, InsertJunctionAndFetch, InsertJunctionRow},
            update::{UpdateLink, UpdateLinkData, UpdateLinkSplit},
        },
    };

    impl<Key, From, To> InsertLinkConsumeData
        for SetId<ManyToMany<Key, From, To>, <To::Id as CollectionId>::IdData>
    where
        To: Collection<Id: SingleColumnId + Identifier> + TableNameExpression + Members + Clone,
        From: Collection<Id: SingleColumnId + Identifier> + TableNameExpression + Clone,
        <From::Id as CollectionId>::IdData: Into<i64>,
        <To::Id as CollectionId>::IdData: Into<i64>,
        Key: Clone + AsRef<str>,
        From: Clone,
        To: Clone,
    {
        type Link = SetId<ManyToMany<Key, From, To>, PhantomData<<To::Id as CollectionId>::IdData>>;

        fn consume_data(
            self,
        ) -> (
            Self::Link,
            InsertLinkData<
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
                    insert_value_data: (),
                    post_op_data: self.id,
                },
            )
        }
    }

    impl<Key, From, To> InsertOneLink
        for SetId<ManyToMany<Key, From, To>, PhantomData<<To::Id as CollectionId>::IdData>>
    where
        To: Collection<Id: SingleColumnId + Identifier> + TableNameExpression + Members + Clone,
        From: Collection<Id: SingleColumnId + Identifier> + TableNameExpression + Clone,
        <From::Id as CollectionId>::IdData: Into<i64>,
        <To::Id as CollectionId>::IdData: Into<i64>,
        Key: Clone + AsRef<str>,
    {
        type PreOp = ();
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
            ConstraintViolation,
        > {
            Ok(((), (), ()))
        }
        type PreOpToInsertValue = ();
        type PreOpToTake = ();
        type PreOpToPostOp = ();
        type InsertNames = ();
        fn insert_names(&self) -> Self::InsertNames {}
        type InsertReturning = ();
        fn insert_returning(&self) -> Self::InsertReturning {}
        type InsertValuesData = ();
        type InsertValues = ();
        fn insert_value(&self, _: Self::InsertValuesData, _: ()) -> Self::InsertValues {}
        type FromRow = OneColumn<&'static str, <From::Id as CollectionId>::IdData>;
        fn from_row(&self) -> Self::FromRow {
            OneColumn {
                as_name: "id",
                as_type: PhantomData,
            }
        }
        type TakeInput = ();
        type PostOp = InsertJunctionAndFetch<Key, From, To>;
        type PostOpOutput = LinkedOutput<<To::Id as CollectionId>::IdData, To::OutputData, ()>;
        fn post_op_output(
            &self,
            poo: <Self::PostOp as OperationOutput>::Output,
        ) -> Result<Self::PostOpOutput, ConstraintViolation> {
            Ok(poo)
        }
        type PostOpData = <To::Id as CollectionId>::IdData;
        fn from_row_result(
            &self,
            to_id: Self::PostOpData,
            from_id: <Self::FromRow as FromRowData>::RData,
            _: Self::PreOpToPostOp,
        ) -> (Self::PostOp, Self::TakeInput) {
            (
                InsertJunctionAndFetch::new(self.relation.clone(), from_id.into(), to_id.into()),
                (),
            )
        }
        type Output = CollectionOutput<<To::Id as CollectionId>::IdData, To::OutputData>;
        fn take(
            self,
            pre_op: LinkedOutput<<To::Id as CollectionId>::IdData, To::OutputData, ()>,
            _: (),
            _: Self::PreOpToTake,
        ) -> Self::Output {
            pre_op.into()
        }
    }

    #[derive(Clone)]
    pub struct SetJunctionId<Key, From, To> {
        pub relation: ManyToMany<Key, From, To>,
        pub from_id: i64,
        pub to_id: i64,
    }

    impl<Key, From, To> UpdateLinkSplit for SetJunctionId<Key, From, To>
    where
        To: Collection<Id: SingleColumnId + Identifier> + TableNameExpression + Members + Clone,
        To::OutputData: Clone,
        <To::Id as CollectionId>::IdData: ::std::convert::From<i64> + Clone,
        From: Collection<Id: SingleColumnId + Identifier + Scoped> + TableNameExpression + Clone,
        Key: Clone + AsRef<str>,
        From: Clone,
        To: Clone,
        ManyToMany<Key, From, To>: Clone,
    {
        type Link = Self;
        fn init_split(
            self,
        ) -> (
            Self::Link,
            UpdateLinkData<
                <Self::Link as UpdateLink>::InitSplitForWheres,
                <Self::Link as UpdateLink>::InitSplitForUpdateValues,
                <Self::Link as UpdateLink>::InitSplitForPreOp,
                <Self::Link as UpdateLink>::InitSplitPostOp,
            >,
        ) {
            let from_id = self.from_id;
            (
                self,
                UpdateLinkData {
                    wheres: (),
                    update_values: from_id,
                    pre_op: (),
                    post_op: (),
                },
            )
        }
    }

    impl<Key, From, To> UpdateLink for SetJunctionId<Key, From, To>
    where
        To: Collection<Id: SingleColumnId + Identifier> + TableNameExpression + Members + Clone,
        From: Collection<Id: SingleColumnId + Identifier + Scoped> + TableNameExpression + Clone,
        ManyToMany<Key, From, To>: Clone,
        Key: Clone + AsRef<str>,
        From: Clone,
        To: Clone,
        To::OutputData: Clone,
        <To::Id as CollectionId>::IdData: ::std::convert::From<i64> + Clone,
    {
        type InitSplitForPreOp = ();
        type PreOpSplitWheres = ();
        type PreOpSplitValues = ();
        type PreOpSplitPostOp = ();
        type PreOpSplitTake = ();
        type PreOp = InsertJunctionRow<Key, From, To>;
        fn pre_op(&self, _: Self::InitSplitForPreOp) -> Self::PreOp {
            InsertJunctionRow::new(self.relation.clone(), self.from_id, self.to_id)
        }
        fn split_pre_op(
            &self,
            _: <Self::PreOp as OperationOutput>::Output,
        ) -> Result<
            (
                Self::PreOpSplitWheres,
                Self::PreOpSplitValues,
                Self::PreOpSplitPostOp,
                Self::PreOpSplitTake,
            ),
            ConstraintViolation,
        > {
            Ok(((), (), (), ()))
        }
        type InitSplitForWheres = ();
        type UpdateWhere = ();
        fn wheres(&self, _: Self::InitSplitForWheres) -> Self::UpdateWhere {}
        type UpdateNames = ();
        fn update_names(&self) -> Self::UpdateNames {}
        type InitSplitForUpdateValues = i64;
        type UpdateValues = UpdatingCol<<From::Id as Identifier>::Identifier, Option<i64>>;
        fn update_values(
            &self,
            values: Self::InitSplitForUpdateValues,
            _: Self::PreOpSplitValues,
        ) -> Self::UpdateValues {
            UpdatingCol {
                col: self.relation.from.id().identifier(),
                set: Some(values),
            }
        }
        type FromRow = ();
        fn from_row(&self) -> Self::FromRow {}
        type PostOp = FetchOne<
            To,
            (),
            ColumnEqual<<To::Id as Identifier>::Identifier, <To::Id as CollectionId>::IdData>,
        >;
        type InitSplitPostOp = ();
        fn post_op(&self, _: Self::InitSplitPostOp, _: Self::PreOpSplitPostOp) -> Self::PostOp {
            FetchOne {
                base: self.relation.to.clone(),
                links: (),
                wheres: ColumnEqual {
                    col: self.relation.to.id().identifier(),
                    eq: <To::Id as CollectionId>::IdData::from(self.to_id),
                },
            }
        }
        fn from_row_result(&self, _: &(), _: &mut Self::PostOp) {}
        type Output = CollectionOutput<<To::Id as CollectionId>::IdData, To::OutputData>;
        type PostOpOutput = LinkedOutput<<To::Id as CollectionId>::IdData, To::OutputData, ()>;
        fn post_op_output(
            &self,
            poo: <Self::PostOp as OperationOutput>::Output,
        ) -> Result<Self::PostOpOutput, ConstraintViolation> {
            Ok(poo.expect("linked row should exist"))
        }
        fn take(
            &self,
            _: (),
            post_op: &mut Self::PostOpOutput,
            _: &mut Self::PreOpSplitTake,
        ) -> Self::Output {
            CollectionOutput {
                id: post_op.id.clone(),
                attributes: post_op.attributes.clone(),
            }
        }
    }

    #[derive(Clone)]
    pub struct RemoveJunctionId<Key, From, To> {
        pub relation: ManyToMany<Key, From, To>,
        pub from_id: i64,
        pub to_id: i64,
    }

    impl<Key, From, To> UpdateLinkSplit for RemoveJunctionId<Key, From, To>
    where
        To: Collection<Id: SingleColumnId + Identifier> + TableNameExpression + Members + Clone,
        To::OutputData: Clone,
        <To::Id as CollectionId>::IdData: ::std::convert::From<i64> + Clone,
        From: Collection<Id: SingleColumnId + Identifier + Scoped> + TableNameExpression + Clone,
        Key: Clone + AsRef<str>,
        From: Clone,
        To: Clone,
        ManyToMany<Key, From, To>: Clone,
    {
        type Link = Self;
        fn init_split(
            self,
        ) -> (
            Self::Link,
            UpdateLinkData<
                <Self::Link as UpdateLink>::InitSplitForWheres,
                <Self::Link as UpdateLink>::InitSplitForUpdateValues,
                <Self::Link as UpdateLink>::InitSplitForPreOp,
                <Self::Link as UpdateLink>::InitSplitPostOp,
            >,
        ) {
            let from_id = self.from_id;
            (
                self,
                UpdateLinkData {
                    wheres: (),
                    update_values: from_id,
                    pre_op: (),
                    post_op: (),
                },
            )
        }
    }

    impl<Key, From, To> UpdateLink for RemoveJunctionId<Key, From, To>
    where
        To: Collection<Id: SingleColumnId + Identifier> + TableNameExpression + Members + Clone,
        From: Collection<Id: SingleColumnId + Identifier + Scoped> + TableNameExpression + Clone,
        ManyToMany<Key, From, To>: Clone,
        Key: Clone + AsRef<str>,
        From: Clone,
        To: Clone,
        To::OutputData: Clone,
        <To::Id as CollectionId>::IdData: ::std::convert::From<i64> + Clone,
    {
        type InitSplitForPreOp = ();
        type PreOpSplitWheres = ();
        type PreOpSplitValues = ();
        type PreOpSplitPostOp = ();
        type PreOpSplitTake =
            Option<LinkedOutput<<To::Id as CollectionId>::IdData, To::OutputData, ()>>;
        type PreOp = FetchOne<
            To,
            (),
            ColumnEqual<<To::Id as Identifier>::Identifier, <To::Id as CollectionId>::IdData>,
        >;
        fn pre_op(&self, _: Self::InitSplitForPreOp) -> Self::PreOp {
            FetchOne {
                base: self.relation.to.clone(),
                links: (),
                wheres: ColumnEqual {
                    col: self.relation.to.id().identifier(),
                    eq: <To::Id as CollectionId>::IdData::from(self.to_id),
                },
            }
        }
        fn split_pre_op(
            &self,
            linked: <Self::PreOp as OperationOutput>::Output,
        ) -> Result<
            (
                Self::PreOpSplitWheres,
                Self::PreOpSplitValues,
                Self::PreOpSplitPostOp,
                Self::PreOpSplitTake,
            ),
            ConstraintViolation,
        > {
            Ok(((), (), (), linked))
        }
        type InitSplitForWheres = ();
        type UpdateWhere = ();
        fn wheres(&self, _: Self::InitSplitForWheres) -> Self::UpdateWhere {}
        type UpdateNames = ();
        fn update_names(&self) -> Self::UpdateNames {}
        type InitSplitForUpdateValues = i64;
        type UpdateValues = UpdatingCol<<From::Id as Identifier>::Identifier, Option<i64>>;
        fn update_values(
            &self,
            values: Self::InitSplitForUpdateValues,
            _: Self::PreOpSplitValues,
        ) -> Self::UpdateValues {
            UpdatingCol {
                col: self.relation.from.id().identifier(),
                set: Some(values),
            }
        }
        type FromRow = ();
        fn from_row(&self) -> Self::FromRow {}
        type PostOp = DeleteJunctionRow<Key, From, To>;
        type InitSplitPostOp = ();
        fn post_op(&self, _: Self::InitSplitPostOp, _: Self::PreOpSplitPostOp) -> Self::PostOp {
            DeleteJunctionRow::new(self.relation.clone(), self.from_id, self.to_id)
        }
        fn from_row_result(&self, _: &(), _: &mut Self::PostOp) {}
        type Output = CollectionOutput<<To::Id as CollectionId>::IdData, To::OutputData>;
        type PostOpOutput = ();
        fn post_op_output(
            &self,
            _: <Self::PostOp as OperationOutput>::Output,
        ) -> Result<Self::PostOpOutput, ConstraintViolation> {
            Ok(())
        }
        fn take(
            &self,
            _: (),
            _: &mut Self::PostOpOutput,
            pre_op_split_take: &mut Self::PreOpSplitTake,
        ) -> Self::Output {
            let linked = pre_op_split_take.take().expect("linked row should exist");
            CollectionOutput {
                id: linked.id,
                attributes: linked.attributes,
            }
        }
    }

    #[derive(Clone)]
    pub struct DeleteManyToManyLinked<Key, From, To> {
        pub link: ManyToMany<Key, From, To>,
        pub from_id: i64,
    }

    impl<Key, From, To> DeleteLinkSplit for DeleteManyToManyLinked<Key, From, To>
    where
        Self: Clone,
        To: Collection,
        From: Collection,
        Key: Clone + AsRef<str>,
        From: Clone,
        To: Clone,
        <From::Id as CollectionId>::IdData: ::std::convert::From<i64> + Copy + Eq + std::hash::Hash,
    {
        type Link = Self;
        type InitSplitForPreOp = ();
        fn init_split(self) -> (Self::Link, Self::InitSplitForPreOp, DeleteLinkData<()>) {
            (self, (), DeleteLinkData { wheres: () })
        }
    }

    impl<Wheres, Key, From, To> DeleteLinkPreOp<Wheres> for DeleteManyToManyLinked<Key, From, To>
    where
        Self: Clone,
        From: Collection<Id: SingleColumnId> + Clone + TableNameExpression,
        To: Collection<Id: SingleColumnId + Identifier> + Clone + TableNameExpression + Members,
        <From as TableNameExpression>::LowerCaseTableNameExpression: AsRef<str>,
        <To as TableNameExpression>::LowerCaseTableNameExpression: AsRef<str>,
        <From::Id as CollectionId>::IdData: ::std::convert::From<i64> + Copy + Eq + std::hash::Hash,
        Wheres: Clone,
        Key: Clone + AsRef<str>,
    {
        type InitSplitForPreOp = ();
        type PreOp = FetchManyToManyLinked<Key, From, To>;
        fn pre_op(&self, _: Self::InitSplitForPreOp, _: &Wheres) -> Self::PreOp {
            FetchManyToManyLinked::new(self.link.clone(), vec![self.from_id.into()])
        }
    }

    impl<Key, From, To> DeleteLink for DeleteManyToManyLinked<Key, From, To>
    where
        Self: Clone,
        To: Collection,
        Key: Clone + AsRef<str>,
        From: Collection + Clone,
        To: Clone,
        <From::Id as CollectionId>::IdData: ::std::convert::From<i64> + Copy + Eq + std::hash::Hash,
    {
        type Output =
            ManyLinkOutput<CollectionOutput<<To::Id as CollectionId>::IdData, To::OutputData>>;
        type PreOpOutput = ManyToManyLinkedMap<
            <From::Id as CollectionId>::IdData,
            <To::Id as CollectionId>::IdData,
            To::OutputData,
        >;
        type PreOpSplitWheres = ();
        type PreOpSplitTake =
            Vec<CollectionOutput<<To::Id as CollectionId>::IdData, To::OutputData>>;
        fn split_pre_op(
            &self,
            mut pre_op: Self::PreOpOutput,
        ) -> (Self::PreOpSplitWheres, Self::PreOpSplitTake) {
            ((), pre_op.remove(&self.from_id.into()).unwrap_or_default())
        }
        type InitSplitForWheres = ();
        type Wheres = ();
        fn wheres(&self, _: Self::InitSplitForWheres, _: Self::PreOpSplitWheres) -> Self::Wheres {}
        type DeleteReturnExpression = ();
        fn delete_return_expression(&self) -> Self::DeleteReturnExpression {}
        type DeleteReturnFromRow = ();
        fn from_row(&self) -> Self::DeleteReturnFromRow {}
        fn take_mut(
            &self,
            _: <Self::DeleteReturnFromRow as FromRowData>::RData,
            pre_op_split_take: &mut Self::PreOpSplitTake,
        ) -> Self::Output {
            ManyLinkOutput {
                many_output: std::mem::take(pre_op_split_take),
            }
        }
        fn take_once(
            &self,
            _: <Self::DeleteReturnFromRow as FromRowData>::RData,
            pre_op_split_take: Self::PreOpSplitTake,
        ) -> Self::Output {
            ManyLinkOutput {
                many_output: pre_op_split_take,
            }
        }
    }
}

pub use crate::operations::fetch_linked_records::FetchManyToManyLinked;
pub use impl_mutate_links::{DeleteManyToManyLinked, RemoveJunctionId, SetJunctionId};

#[cfg(test)]
mod test {
    use sqlx::Sqlite;

    use crate::{
        collections::Collection,
        connect_in_memory::ConnectInMemory,
        expressions::ColumnEqual,
        extentions::common_expressions::Scoped,
        links::{DefaultRelationKey, relation_many_to_many::ManyToMany},
        on_migrate::OnMigrate,
        operations::{
            CollectionOutput, LinkedOutput, ManyLinkOutput, Operation,
            fetch_many::{FetchMany, ManyOutput},
            fetch_one::FetchOne,
        },
        sqlx_query_builder::{Expression, StatementBuilder},
        test_module::{self, Category, Tag, Todo},
    };

    fn todo_to_tag_link() -> ManyToMany<DefaultRelationKey, test_module::todo, test_module::tag> {
        ManyToMany {
            relation_key: DefaultRelationKey,
            from: test_module::todo,
            to: test_module::tag,
        }
    }

    fn category_to_tag_link()
    -> ManyToMany<DefaultRelationKey, test_module::category, test_module::tag> {
        ManyToMany {
            relation_key: DefaultRelationKey,
            from: test_module::category,
            to: test_module::tag,
        }
    }

    async fn migrate_todo_tag_fixtures(
        conn: &mut sqlx::SqliteConnection,
        link: &ManyToMany<DefaultRelationKey, test_module::todo, test_module::tag>,
    ) {
        sqlx::query(
            r#"
            CREATE TABLE "Tag" ("id" INTEGER PRIMARY KEY AUTOINCREMENT, "title" TEXT NOT NULL);
            CREATE TABLE "Todo" ("id" INTEGER PRIMARY KEY AUTOINCREMENT, "title" TEXT NOT NULL, "done" BOOLEAN NOT NULL, "description" TEXT);
            "#,
        )
        .execute(&mut *conn)
        .await
        .unwrap();

        let mut qb = StatementBuilder::<Sqlite>::default();
        link.statments().expression(&mut qb);
        sqlx::query(&qb.stmt).execute(&mut *conn).await.unwrap();
    }

    async fn migrate_category_tag_fixtures(
        conn: &mut sqlx::SqliteConnection,
        link: &ManyToMany<DefaultRelationKey, test_module::category, test_module::tag>,
    ) {
        sqlx::query(
            r#"
            CREATE TABLE "Tag" ("id" INTEGER PRIMARY KEY AUTOINCREMENT, "title" TEXT NOT NULL);
            CREATE TABLE "Category" ("id" INTEGER PRIMARY KEY AUTOINCREMENT, "title" TEXT NOT NULL);
            "#,
        )
        .execute(&mut *conn)
        .await
        .unwrap();

        let mut qb = StatementBuilder::<Sqlite>::default();
        link.statments().expression(&mut qb);
        sqlx::query(&qb.stmt).execute(&mut *conn).await.unwrap();
    }

    #[test]
    fn migrate_statement_creates_junction_table() {
        let link = todo_to_tag_link();
        let mut qb = StatementBuilder::<Sqlite>::default();
        link.statments().expression(&mut qb);

        pretty_assertions::assert_eq!(
            qb.stmt,
            r#"CREATE TABLE "ct_todotag_def" ("todo_id" INTEGER NOT NULL  REFERENCES "Todo"("id") ON DELETE CASCADE, "tag_id" INTEGER NOT NULL  REFERENCES "Tag"("id") ON DELETE CASCADE, PRIMARY KEY ("todo_id", "tag_id"));"#
        );
    }

    #[tokio::test]
    async fn fetch_many_returns_one_row_per_todo_with_all_tags() {
        let mut conn = Sqlite::in_memory_connection().await;

        let link = todo_to_tag_link();
        migrate_todo_tag_fixtures(&mut conn, &link).await;

        sqlx::query(
            r#"
            INSERT INTO "Tag" ("title") VALUES ('urgent'), ('home');
            INSERT INTO "Todo" ("title", "done", "description") VALUES
                ('todo_a', true, 'a'),
                ('todo_b', false, 'b');
            INSERT INTO "ct_todotag_def" ("todo_id", "tag_id") VALUES
                (1, 1),
                (1, 2),
                (2, 1);
            "#,
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let output = Operation::<Sqlite>::exec_operation(
            FetchMany {
                base: test_module::todo,
                wheres: (),
                links: link,
                cursor_order_by: test_module::todo_members::id,
                cursor_first_item: None::<(i64, ())>,
                limit: 10,
            },
            &mut conn,
        )
        .await;

        pretty_assertions::assert_eq!(
            output,
            ManyOutput {
                items: vec![
                    LinkedOutput {
                        id: 1,
                        attributes: Todo {
                            title: "todo_a".to_string(),
                            done: true,
                            description: Some("a".to_string()),
                        },
                        links: ManyLinkOutput {
                            many_output: vec![
                                CollectionOutput {
                                    id: 1,
                                    attributes: Tag {
                                        title: "urgent".to_string(),
                                    },
                                },
                                CollectionOutput {
                                    id: 2,
                                    attributes: Tag {
                                        title: "home".to_string(),
                                    },
                                },
                            ],
                        },
                    },
                    LinkedOutput {
                        id: 2,
                        attributes: Todo {
                            title: "todo_b".to_string(),
                            done: false,
                            description: Some("b".to_string()),
                        },
                        links: ManyLinkOutput {
                            many_output: vec![CollectionOutput {
                                id: 1,
                                attributes: Tag {
                                    title: "urgent".to_string(),
                                },
                            },],
                        },
                    },
                ],
                next_item: None,
            }
        );
    }

    #[tokio::test]
    async fn fetch_one_returns_all_linked_tags() {
        let mut conn = Sqlite::in_memory_connection().await;

        let link = todo_to_tag_link();
        migrate_todo_tag_fixtures(&mut conn, &link).await;

        sqlx::query(
            r#"
            INSERT INTO "Tag" ("title") VALUES ('urgent'), ('home');
            INSERT INTO "Todo" ("title", "done", "description") VALUES
                ('todo_a', true, 'a'),
                ('todo_b', false, 'b');
            INSERT INTO "ct_todotag_def" ("todo_id", "tag_id") VALUES
                (1, 1),
                (1, 2),
                (2, 1);
            "#,
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let output = Operation::<Sqlite>::exec_operation(
            FetchOne {
                base: test_module::todo,
                wheres: ColumnEqual {
                    col: test_module::todo.id().scoped(),
                    eq: 1,
                },
                links: link,
            },
            &mut conn,
        )
        .await;

        pretty_assertions::assert_eq!(
            output,
            Some(LinkedOutput {
                id: 1,
                attributes: Todo {
                    title: "todo_a".to_string(),
                    done: true,
                    description: Some("a".to_string()),
                },
                links: ManyLinkOutput {
                    many_output: vec![
                        CollectionOutput {
                            id: 1,
                            attributes: Tag {
                                title: "urgent".to_string(),
                            },
                        },
                        CollectionOutput {
                            id: 2,
                            attributes: Tag {
                                title: "home".to_string(),
                            },
                        },
                    ],
                },
            })
        );
    }

    #[tokio::test]
    async fn fetch_many_from_category_returns_one_row_per_category_with_all_tags() {
        let mut conn = Sqlite::in_memory_connection().await;

        let link = category_to_tag_link();
        migrate_category_tag_fixtures(&mut conn, &link).await;

        sqlx::query(
            r#"
            INSERT INTO "Tag" ("title") VALUES ('urgent'), ('review');
            INSERT INTO "Category" ("title") VALUES ('work'), ('personal');
            INSERT INTO "ct_categorytag_def" ("category_id", "tag_id") VALUES
                (1, 1),
                (1, 2),
                (2, 2);
            "#,
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let output = Operation::<Sqlite>::exec_operation(
            FetchMany {
                base: test_module::category,
                wheres: (),
                links: link,
                cursor_order_by: test_module::category_members::id,
                cursor_first_item: None::<(i64, ())>,
                limit: 10,
            },
            &mut conn,
        )
        .await;

        pretty_assertions::assert_eq!(
            output,
            ManyOutput {
                items: vec![
                    LinkedOutput {
                        id: 1,
                        attributes: Category {
                            title: "work".to_string(),
                        },
                        links: ManyLinkOutput {
                            many_output: vec![
                                CollectionOutput {
                                    id: 1,
                                    attributes: Tag {
                                        title: "urgent".to_string(),
                                    },
                                },
                                CollectionOutput {
                                    id: 2,
                                    attributes: Tag {
                                        title: "review".to_string(),
                                    },
                                },
                            ],
                        },
                    },
                    LinkedOutput {
                        id: 2,
                        attributes: Category {
                            title: "personal".to_string(),
                        },
                        links: ManyLinkOutput {
                            many_output: vec![CollectionOutput {
                                id: 2,
                                attributes: Tag {
                                    title: "review".to_string(),
                                },
                            },],
                        },
                    },
                ],
                next_item: None,
            }
        );
    }

    #[tokio::test]
    async fn fetch_many_without_links_returns_single_row_with_empty_links() {
        let mut conn = Sqlite::in_memory_connection().await;

        let link = todo_to_tag_link();
        migrate_todo_tag_fixtures(&mut conn, &link).await;

        sqlx::query(
            r#"
            INSERT INTO "Todo" ("title", "done", "description") VALUES ('lonely', false, NULL);
            "#,
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let output = Operation::<Sqlite>::exec_operation(
            FetchMany {
                base: test_module::todo,
                wheres: (),
                links: link,
                cursor_order_by: test_module::todo_members::id,
                cursor_first_item: None::<(i64, ())>,
                limit: 10,
            },
            &mut conn,
        )
        .await;

        pretty_assertions::assert_eq!(
            output,
            ManyOutput {
                items: vec![LinkedOutput {
                    id: 1,
                    attributes: Todo {
                        title: "lonely".to_string(),
                        done: false,
                        description: None,
                    },
                    links: ManyLinkOutput {
                        many_output: vec![]
                    },
                },],
                next_item: None,
            }
        );
    }

    #[tokio::test]
    async fn update_set_junction_id_links_tag() {
        use super::SetJunctionId;
        use crate::{
            operations::update::Update,
            test_module::{Tag, TodoPartial},
            update_mod::Update as PartialUpdate,
        };

        let mut conn = Sqlite::in_memory_connection().await;

        let link = todo_to_tag_link();
        migrate_todo_tag_fixtures(&mut conn, &link).await;

        sqlx::query(
            r#"
            INSERT INTO "Tag" ("title") VALUES ('urgent');
            INSERT INTO "Todo" ("title", "done", "description") VALUES ('todo', false, 'before');
            "#,
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let out = Operation::<Sqlite>::exec_operation(
            Update {
                base: test_module::todo,
                partial: TodoPartial {
                    title: PartialUpdate::Set("linked".to_string()),
                    done: PartialUpdate::Keep,
                    description: PartialUpdate::Keep,
                },
                wheres: ColumnEqual {
                    col: test_module::todo.id().scoped(),
                    eq: 1,
                },
                links: SetJunctionId {
                    relation: link,
                    from_id: 1,
                    to_id: 1,
                },
            },
            &mut conn,
        )
        .await
        .unwrap();

        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].links,
            CollectionOutput {
                id: 1,
                attributes: Tag {
                    title: "urgent".to_string(),
                },
            }
        );
    }
}
