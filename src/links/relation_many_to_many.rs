use crate::QueryBuilder;
use crate::collections::Collection;
use crate::{
    collections::OnMigrate,
    operations::{SimpleOutput, select_one::SelectOneFragment},
    prelude::stmt::SelectSt,
};
use sqlx::{Sqlite, sqlite::SqliteRow};

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
    T2::Yeild: Send + Sync,
    T1: Send + Sync,
{
    type Inner = (Option<i32>, Vec<SimpleOutput<T2::Yeild>>);

    type Output = Vec<SimpleOutput<T2::Yeild>>;

    fn on_select(&self, _data: &mut Self::Inner, _st: &mut SelectSt<Sqlite>) {
        // no op
    }

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
        use sqlx::Row;

        let id: i32 = row.get("local_id");
        data.0 = Some(id);
    }

    fn sub_op<'this>(
        &'this self,
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
            .map(|r| SimpleOutput {
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
