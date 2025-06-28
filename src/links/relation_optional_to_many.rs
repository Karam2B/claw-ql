use crate::Accept;
use crate::execute::Execute;
use crate::operations::CollectionOutput;
use crate::operations::delete_one_op::DeleteOneFragment;
use crate::prelude::join::left_join;
use crate::{
    QueryBuilder,
    migration::OnMigrate,
    operations::{collections::Collection, select_one_op::SelectOneFragment},
    prelude::{col, join, stmt::SelectSt},
};
use convert_case::{Case, Casing};
use sqlx::Pool;
use sqlx::{ColumnIndex, Decode, Executor, Row, Sqlite, prelude::Type};

#[derive(Clone)]
pub struct OptionalToMany<F, T> {
    pub foriegn_key: String,
    pub from: F,
    pub to: T,
}

#[derive(Clone)]
pub struct OptionalToManyInverse<F, T> {
    pub foriegn_key: String,
    pub from: F,
    pub to: T,
}

impl<From, To> OnMigrate<Sqlite> for OptionalToMany<From, To>
where
    From: Collection<Sqlite>,
    To: Collection<Sqlite>,
{
    async fn custom_migration<'e>(
        &self,
        exec: impl for<'q> Executor<'q, Database = Sqlite> + Clone,
    ) {
        sqlx::query(&format!(
            "
ALTER TABLE {from_table_name} 
ADD COLUMN {col_name} INT
REFERENCES {to_table_name} (id)
{dio}
ON DELETE SET NULL;
",
            from_table_name = self.from.table_name(),
            to_table_name = self.to.table_name(),
            col_name = format!("{}_id", self.to.table_name().to_case(Case::Snake)),
            dio = ""
        ))
        .execute(exec.clone())
        .await
        .unwrap();
    }
}

impl<S, From, To> SelectOneFragment<S> for OptionalToMany<From, To>
where
    S: QueryBuilder,
    To: Collection<S, Data: Send + Sync>,
    From: Collection<S, Data: Send + Sync>,
    for<'c> &'c str: ColumnIndex<S::Row>,
    for<'q> i64: Decode<'q, S>,
    i64: Type<S>,
{
    type Output = Option<CollectionOutput<To::Data>>;
    type Inner = Option<(i64, To::Data)>;

    fn on_select(&mut self, _: &mut Self::Inner, st: &mut SelectSt<S>) {
        st.join(join::left_join {
            foriegn_table: self.to.table_name().to_string(),
            foriegn_column: "id".to_string(),
            local_column: self.foriegn_key.to_string(),
        });
        st.select(col(&self.foriegn_key).table(self.from.table_name()));
        self.to.on_select(st);
    }

    fn from_row(&mut self, data: &mut Self::Inner, row: &S::Row) {
        let id: Option<i64> = row.get(self.foriegn_key.as_str());
        if let Some(id) = id {
            let value = self.to.from_row_scoped(row);
            *data = Some((id, value));
        }
    }

    async fn sub_op<'this>(&'this mut self, _: &'this mut Self::Inner, _: Pool<S>) {
        // no sub_op for optional_to_many
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        data.map(|(id, attr)| CollectionOutput { id, attr })
    }
}

impl<S, From, To> DeleteOneFragment<S> for OptionalToMany<From, To>
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

#[cfg(feature = "serde")]
mod dynamic_client {
    use serde::Serialize;
    use sqlx::{ColumnIndex, Decode, prelude::Type};

    use crate::{
        QueryBuilder,
        collections::{Collection, CollectionBasic},
        json_client::RuntimeResult,
        links::relation::{DynamicLinkForRelation, RelationEntry, RelationType},
        prelude::macro_relation::{OptionalToMany, OptionalToManyInverse},
    };

    impl<F, T> DynamicLinkForRelation for OptionalToManyInverse<F, T>
    where
        F: CollectionBasic,
        T: CollectionBasic,
    {
        fn metadata(&self) -> crate::links::relation::RelationEntry {
            struct OptionalToManyInverseIdent;
            static OPTIONAL_TO_MANY_INVERSE: OptionalToManyInverseIdent =
                OptionalToManyInverseIdent;
            impl RelationType for OptionalToManyInverseIdent {
                fn inspect(self: &'static Self) -> &'static str {
                    "optional_to_many_inverse"
                }
            }

            RelationEntry {
                from: self.from.table_name().to_owned(),
                to: self.to.table_name().to_owned(),
                ty: &OPTIONAL_TO_MANY_INVERSE,
            }
        }
    }
    impl<F, T> DynamicLinkForRelation for OptionalToMany<F, T>
    where
        F: CollectionBasic,
        T: CollectionBasic,
    {
        fn metadata(&self) -> crate::links::relation::RelationEntry {
            struct OptionalToManyIdent;
            static OPTIONAL_TO_MANY: OptionalToManyIdent = OptionalToManyIdent;
            impl RelationType for OptionalToManyIdent {
                fn inspect(self: &'static Self) -> &'static str {
                    "optional_to_many"
                }
            }

            RelationEntry {
                from: self.from.table_name().to_owned(),
                to: self.to.table_name().to_owned(),
                ty: &OPTIONAL_TO_MANY,
            }
        }
    }
}
