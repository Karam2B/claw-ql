use crate::QueryBuilder;
use crate::collections::Collection;
use crate::operations::CollectionOutput;
use crate::{
    collections::OnMigrate, operations::select_one_op::SelectOneFragment, prelude::stmt::SelectSt,
};
use serde::Serialize;
use sqlx::{Sqlite, sqlite::SqliteRow};
use std::ops::Not;

#[derive(Clone)]
pub struct ManyToMany<T1, T2> {
    pub junction: String,
    pub table_1: T1,
    pub id_1: String,
    pub table_2: T2,
    pub id_2: String,
}

impl<T1, T2> OnMigrate<Sqlite> for ManyToMany<T1, T2>
where
    T1: Collection<Sqlite>,
    T2: Collection<Sqlite>,
{
    fn custom_migration<'e>(
        &self,
        exec: impl for<'q> sqlx::Executor<'q, Database = Sqlite> + Clone,
    ) -> impl Future<Output = ()>
    where
        Sqlite: QueryBuilder,
    {
        async {
            let ManyToMany {
                junction,
                table_1,
                id_1,
                table_2,
                id_2,
            }: &ManyToMany<T1, T2> = self;

            sqlx::query(&format!(
                "
            CREATE TABLE {junction} (
                {id_1} INTEGER NOT NULL, 
                {id_2} INTEGER NOT NULL,
                PRIMARY KEY ({id_1}, {id_2}),
                FOREIGN KEY ({id_1}) REFERENCES {table_1}(id) ON DELETE CASCADE,
                FOREIGN KEY ({id_2}) REFERENCES {table_2}(id) ON DELETE CASCADE
            )",
                table_1 = table_1.table_name(),
                table_2 = table_2.table_name()
            ))
            .execute(exec)
            .await
            .unwrap();
        }
    }
}

impl<T1, T2> SelectOneFragment<Sqlite> for ManyToMany<T1, T2>
where
    T2: Send + Sync + Collection<Sqlite>,
    T2::Data: Send + Sync,
    T1: Send + Sync,
{
    type Inner = (Option<i32>, Vec<CollectionOutput<T2::Data>>);

    type Output = Vec<CollectionOutput<T2::Data>>;

    fn on_select(&mut self, _data: &mut Self::Inner, _st: &mut SelectSt<Sqlite>) {
        // no op
    }

    fn from_row(&mut self, data: &mut Self::Inner, row: &SqliteRow) {
        use sqlx::Row;

        let id: i32 = row.get("local_id");
        data.0 = Some(id);
    }

    fn sub_op<'this>(
        &'this mut self,
        data: &'this mut Self::Inner,
        pool: sqlx::Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + use<'this, T1, T2> {
        async move {
            use sqlx::Row;
            let res = sqlx::query(&format!(
                "SELECT * FROM {jun} AS B LEFT JOIN {target} AS L ON B.{} = L.id WHERE B.{} = $1",
                self.id_2,
                self.id_1,
                jun = self.junction,
                target = self.table_2.table_name(),
            ))
            .bind(data.0.unwrap())
            .fetch_all(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|r| CollectionOutput {
                attr: self.table_2.from_row_noscope(&r),
                id: r.get("id"),
            })
            .collect();

            data.1 = res;
        }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        data.1
    }
}

// impl<S: QueryBuilder, T0, T1> DynamicLinkForRelation<S> for ManyToMany<T0, T1>
// where
//     Self: Clone,
//     T0: 'static,
//     T1: 'static,
//     ManyToMany<T0, T1>: SelectOneFragment<S, Output: Serialize, Inner: 'static>,
// {
//     fn global_ident(&self) -> &'static str {
//         "many_to_many"
//     }
//
//     fn on_each_select_one_request(
//         &self,
//         input: serde_json::Value,
//     ) -> Result<Box<dyn SelectOneJsonFragment<S>>, String> {
//         if input.is_object().not() {
//             return Err("many_to_many relation is only input is {}".to_string());
//         }
//         let this = self.clone();
//
//         Ok(Box::new((this, Default::default())))
//     }
// }
