use claw_ql::execute::Executable;
use serde_json::Value;
use serde_json::json;
use sqlx::Executor;
use sqlx::Sqlite;
use sqlx::sqlite::SqliteRow;
use std::marker::PhantomData;

#[track_caller]
fn rows_to_json(rows: Vec<SqliteRow>) -> serde_json::Value {
    let mut out = vec![];

    for row in rows {
        let mut entry = serde_json::Map::default();
        use sqlx::Row;
        let cols = row.columns();

        for col in cols {
            use sqlx::Column;
            use sqlx::TypeInfo;
            let name = col.name();
            let s = col.type_info();
            match s.name() {
                "NULL" => {
                    let _ = entry.insert(name.to_string(), json!(null));
                }
                "TEXT" => {
                    entry.insert(name.to_string(), Value::String(row.get(name)));
                }
                "INTEGER" => {
                    let v: i64 = row.get(name);
                    entry.insert(name.to_string(), v.into());
                }
                "REAL" | "NUMBERIC" => {
                    let v: f64 = row.get(name);
                    entry.insert(name.to_string(), v.into());
                }
                _ => {
                    panic!("{} is not supported", s.name())
                }
            }
        }
        out.push(Value::Object(entry));
    }

    Value::Array(out)
}

#[tokio::test]
async fn main() {
    let pool = Sqlite::connect_in_memory().await;

    let r = pool
        .fetch_all(Executable {
            string: "SELECT 1;",
            arguments: Default::default(),
            db: PhantomData,
        })
        .await
        .unwrap();

    let s = rows_to_json(r);

    pretty_assertions::assert_eq!(s, json!(null));
}
