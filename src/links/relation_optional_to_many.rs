#![allow(unexpected_cfgs)]

use crate::links::{LinkedToBase, LinkedViaId};

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct optional_to_many<Id, F, T> {
    pub foriegn_key: Id,
    pub from: F,
    pub to: T,
}

impl<Id, F, T> LinkedViaId for optional_to_many<Id, F, T> {}

impl<Id, F, T> LinkedToBase for optional_to_many<Id, F, T> {
    type Base = F;
}

mod impl_on_migrate {
    use crate::{
        collections::{Collection, Id},
        expressions::{
            col_def, foriegn_key, id_constraint, on_delete_set_null, sqlx_type, table_as_expression,
        },
        links::relation_optional_to_many::optional_to_many,
        on_migrate::OnMigrate,
        statements::AddColumn,
    };

    impl<F: Collection + Clone, T: Collection<Id: Id<SqlIdent: ToString>>> OnMigrate
        for optional_to_many<String, F, T>
    {
        type Statements = AddColumn<
            table_as_expression<F>,
            col_def<
                String,
                sqlx_type<Option<i64>>,
                (id_constraint<String, foriegn_key<(on_delete_set_null,)>>,),
            >,
        >;
        fn statments(&self) -> Self::Statements {
            AddColumn {
                table: table_as_expression(self.from.clone()),
                col_def: col_def {
                    name: self.foriegn_key.to_string(),
                    ty: sqlx_type::default(),
                    constraints: (id_constraint(
                        format!("fk_{}", self.foriegn_key),
                        foriegn_key {
                            references_table: self.to.table_name().to_string(),
                            references_col: self.to.id().ident().to_string(),
                            ons: (on_delete_set_null,),
                        },
                    ),),
                },
            }
        }
    }
}

mod impl_fetch_one {
    use sqlx::{ColumnIndex, Database, Row, Type};

    use crate::{
        collections::{Collection, SingleIncremintalInt},
        expressions::{left_join, scoped_column, table},
        extentions::Members,
        from_row::{FromRowAlias, pre_alias},
        links::relation_optional_to_many::optional_to_many,
        operations::{
            CollectionOutput, Operation,
            fetch_one::{LinkFetchOne, SelectStatementExtendableParts},
        },
    };

    impl<S, F, T> LinkFetchOne<S> for optional_to_many<String, F, T>
    where
        T: Collection<Id = SingleIncremintalInt> + Members<S>,
        T: for<'r> FromRowAlias<'r, <S as Database>::Row>,
        F: Collection,
        S: Database,
        i64: for<'q> sqlx::Decode<'q, S> + Type<S>,
        for<'q> &'q str: ColumnIndex<S::Row>,
    {
        type Joins = (left_join,);

        type Wheres = ();

        fn extend_select(
            &self,
        ) -> SelectStatementExtendableParts<
            //
            Vec<scoped_column<String, String>>,
            Self::Joins,
            Self::Wheres,
        > {
            let mut to_members =
                vec![table(self.to.table_name().to_string()).col("id".to_string())];

            to_members.extend(
                Members::members_names(&self.to)
                    .into_iter()
                    .map(|e| return table(self.to.table_name().to_string()).col(e.to_string())),
            );

            SelectStatementExtendableParts {
                non_aggregating_select_items: to_members,
                non_duplicating_joins: (left_join {
                    ft: self.to.table_name().to_string(),
                    fc: "id".to_string(),
                    lt: self.from.table_name().to_string(),
                    lc: self.foriegn_key.clone(),
                },),
                wheres: (),
            }
        }

        type Inner = CollectionOutput<i64, T::Data>;

        type SubOp = ();

        fn sub_op(&self, row: pre_alias<<S as sqlx::Database>::Row>) -> (Self::SubOp, Self::Inner)
        where
            S: sqlx::Database,
        {
            (
                (),
                CollectionOutput {
                    id: row.0.get(format!("{}id", row.1).as_str()),
                    attributes: self.to.pre_alias(row).unwrap(),
                },
            )
        }

        type Output = CollectionOutput<i64, T::Data>;

        fn take(
            self,
            _: <Self::SubOp as Operation<S>>::Output,
            inner: Self::Inner,
        ) -> Self::Output {
            inner
        }
    }
}

pub mod impl_dynamic_link {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    use crate::{
        collections::CollectionBasic,
        links::{CollectionsStore, DynamicLink, relation_optional_to_many::optional_to_many},
    };

    pub struct OptionalToManyLinks<DynamicBase> {
        pub all: Vec<optional_to_many<String, DynamicBase, DynamicBase>>,
    }

    #[derive(Deserialize)]
    pub struct RelationOnRequest {
        to: String,
        #[serde(default = "set_def")]
        id: String,
    }

    #[derive(Serialize)]
    pub struct RelationNotFound {
        pub expected_id: String,
        pub expected_from: String,
        pub expected_to: String,
    }

    fn set_def() -> String {
        "default".to_string()
    }

    impl<
        DynamicBase: CollectionBasic + Clone + CollectionsStore<Store = HashMap<String, DynamicBase>>,
        S: 'static,
    > DynamicLink<DynamicBase, S> for OptionalToManyLinks<DynamicBase>
    {
        type OnRequest = optional_to_many<String, DynamicBase, DynamicBase>;

        type OnRequestInput = RelationOnRequest;

        type OnRequestError = RelationNotFound;

        type CreateLinkOk = ();

        type CreateLinkInput = ();

        type CreateLinkError = ();

        type ModifyLinkOk = ();

        type ModifyLinkInput = ();

        type ModifyLinkError = ();

        fn on_request(
            &self,
            base: DynamicBase,
            input: Self::OnRequestInput,
        ) -> Result<Self::OnRequest, Self::OnRequestError> {
            self.all
                .iter()
                .find(|e| {
                    e.foriegn_key == input.id
                        && e.from.table_name_lower_case() == base.table_name_lower_case()
                        && e.to.table_name_lower_case() == input.to
                })
                .cloned()
                .ok_or_else(|| RelationNotFound {
                    expected_id: input.id,
                    expected_from: base.table_name_lower_case().to_string(),
                    expected_to: input.to,
                })
        }

        fn create_link(
            &self,
            _: &HashMap<String, DynamicBase>,
            _: Self::CreateLinkInput,
        ) -> Result<Self::CreateLinkOk, Self::CreateLinkError> {
            todo!()
        }

        fn modify_link(
            &self,
            _: &HashMap<String, DynamicBase>,
            _: Self::ModifyLinkInput,
        ) -> Result<Self::ModifyLinkOk, Self::ModifyLinkError> {
            todo!()
        }
    }
}

#[cfg(feature = "skip_without_comment")]
mod old_api_waiting_refactoring {
    impl<S, From, To> DeleteOneFragment<S> for optional_to_many<From, To>
    where
        From: Collection<S, Data: Sync + Send>,
        To: Collection<S, Data: Sync + Send>,
        S: QueryBuilder + Accept<i64>,
        for<'s> &'s str: ColumnIndex<S::Row>,
        SelectSt<S>: Execute<S>,
        S::Fragment: Send,
        S::Context1: Send,
        i64: for<'d> sqlx::Decode<'d, S> + Type<S>,
    {
        type Output = Option<CollectionOutput<To::Data>>;

        type Inner = Option<CollectionOutput<To::Data>>;

        async fn first_sup_op<'this, E: for<'q> Executor<'q, Database = S> + Clone>(
            &'this mut self,
            data: &'this mut Self::Inner,
            exec: E,
            id: i64,
        ) {
            use sqlx::Row;
            let mut st = SelectSt::init(self.from.table_name());
            self.to.on_select(&mut st);
            st.join(left_join {
                local_column: self.foriegn_key.clone(),
                foriegn_column: "id".to_string(),
                foriegn_table: self.to.table_name().to_string(),
            });
            let alias = format!("{}_id", self.to.table_name());
            st.select(col("id").table(self.to.table_name()).alias(&alias));
            st.where_(col("id").table(self.from.table_name()).eq(id));

            *data = st
                .fetch_optional(exec, |r| {
                    Ok(CollectionOutput {
                        attr: self.to.from_row_scoped(&r),
                        id: r.get(&*alias),
                    })
                })
                .await
                .unwrap();
        }

        fn returning(&self) -> Vec<String> {
            vec![]
        }

        fn from_row(&mut self, _data: &mut Self::Inner, _row: &S::Row) {
            /* no-op */
        }

        fn take(self, data: Self::Inner) -> Self::Output {
            data
        }
    }
}
