use serde_json::Value;
use sqlx::TypeInfo;
use sqlx::{
    Column, ColumnIndex, Database, Execute, IntoArguments, Row, Sqlite, sqlite::SqliteTypeInfo,
};

pub trait ToJson {
    type Database: Database;
    fn get_from_row(&self, row: &<Self::Database as Database>::Row, ordinal: usize) -> Value;
}

impl ToJson for SqliteTypeInfo {
    type Database = Sqlite;
    fn get_from_row(&self, row: &<Self::Database as Database>::Row, o: usize) -> Value {
        match self.name() {
            "NULL" => return Value::Null,
            "TEXT" => return Value::String(row.get::<'_, String, _>(o)),
            "REAL" => {
                return Value::Number(
                    serde_json::Number::from_f64(row.get::<'_, f64, _>(o)).unwrap(),
                );
            }
            "BLOB" => {
                panic!("BLOB is not supported")
            }
            "INTEGER" => {
                return Value::Number(row.get::<'_, i64, _>(o).into());
            }
            "NUMERIC" => {
                return Value::Number(row.get::<'_, i64, _>(o).into());
            }

            // non-standard extensions
            "BOOLEAN" => {
                return Value::Bool(row.get::<'_, bool, _>(o));
            }
            "DATE" => {
                return Value::String(row.get::<'_, String, _>(o));
            }
            "TIME" => {
                return Value::String(row.get::<'_, String, _>(o));
            }
            "DATETIME" => {
                return Value::String(row.get::<'_, String, _>(o));
            }
            _ => {
                panic!("breaking changes in SqliteTypeInfo")
            }
        }
    }
}

pub async fn fetch_one_as_json<S: Database>(
    str: &str,
    exec: impl sqlx::Executor<'_, Database = S>,
) -> sqlx::Result<serde_json::Value>
where
    for<'q> <S as sqlx::Database>::Arguments<'q>: IntoArguments<'q, S>,
    S::TypeInfo: ToJson<Database = S>,
    usize: ColumnIndex<<S as sqlx::Database>::Row>,
{
    let row: S::Row = sqlx::query(str).fetch_one(exec).await?;

    if row.columns().len() == 0 {
        return Ok(Value::Null);
    }

    let mut ret = serde_json::Map::default();

    for col in row.columns().iter() {
        let value = col.type_info().get_from_row(&row, col.ordinal());
        let s = ret.insert(col.name().to_owned(), value);
        if s.is_some() {
            panic!("column names should be unique: found dup {}", col.name())
        }
    }

    Ok(Value::Object(ret))
}

pub async fn fetch_many_as_json<S: Database>(
    str: &str,
    exec: impl sqlx::Executor<'_, Database = S>,
) -> sqlx::Result<serde_json::Value>
where
    for<'q> <S as sqlx::Database>::Arguments<'q>: IntoArguments<'q, S>,
    S::TypeInfo: ToJson<Database = S>,
    usize: ColumnIndex<<S as sqlx::Database>::Row>,
{
    let rows: Vec<S::Row> = sqlx::query(str).fetch_all(exec).await?;

    if rows.len() == 0 {
        return Ok(Value::Null);
    }
    if rows[0].columns().len() == 0 {
        return Ok(Value::Null);
    }

    let mut ret = Vec::new();
    let mut info: Vec<(&str, &dyn ToJson<Database = S>, usize)> = Vec::new();

    for col in rows[0].columns().iter() {
        info.push((col.name(), col.type_info(), col.ordinal()));
    }

    for row in rows.iter() {
        let mut map = serde_json::Map::default();
        for info in info.iter() {
            map.insert(info.0.to_owned(), info.1.get_from_row(&row, info.2));
        }
        ret.push(serde_json::Value::Object(map));
    }

    Ok(Value::Array(ret))
}
