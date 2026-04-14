use sqlx::{Database, Row};

pub fn inspect_row<S: Row>(row: &S) -> String {
    use sqlx::Column;
    use sqlx::TypeInfo;
    let mut map = Vec::new();
    for (name, type_name) in row
        .columns()
        .iter()
        .map(|cc| (cc.name(), cc.type_info().name()))
    {
        map.push(format!("{}: {}", name, type_name));
    }
    map.join("\n")
}

pub trait RowToJson: Database {
    type RowInfo;
    fn row_info(row: &Self::Row) -> Self::RowInfo;
    fn vec_to_json_ref(v: Vec<Self::Row>, info: &Self::RowInfo) -> serde_json::Value;
    fn vec_to_json(v: Vec<Self::Row>) -> serde_json::Value
    where
        Self: Sized;
    fn to_json(row: &Self::Row) -> serde_json::Value;
}

mod impl_sqlite {
    use super::*;
    use serde_json::Value;
    use serde_json::json;
    use sqlx::Column;
    use sqlx::Row;
    use sqlx::Sqlite;
    use sqlx::TypeInfo;
    use sqlx::sqlite::SqliteRow;

    #[derive(Debug, PartialEq, Eq)]
    pub enum TypeName {
        Text,
        Integer,
        Real,
        Numeric,
    }

    impl RowToJson for Sqlite {
        type RowInfo = Vec<(
            String,
            bool, // nullable
            TypeName,
        )>;
        fn row_info(row: &SqliteRow) -> Self::RowInfo {
            let cols = row.columns();
            let mut info = Vec::new();
            for col in cols {
                info.push((
                    col.name().to_string(),
                    col.type_info().is_null(),
                    match col.type_info().name() {
                        "NULL" => panic!("what the fuck is null? the way sqlx/sqlite implements TypeInfo makes RowToJson unimplementable"),
                        "TEXT" => TypeName::Text,
                        "INTEGER" => TypeName::Integer,
                        "REAL" | "NUMBERIC" => TypeName::Real,
                        _ => panic!("{} is not supported", col.type_info().name()),
                    },
                ));
            }
            info
        }
        fn vec_to_json(v: Vec<SqliteRow>) -> serde_json::Value
        where
            Self: Sized,
        {
            let first = match v.first() {
                Some(row) => row,
                None => return json!([]),
            };
            let info = Self::row_info(first);
            Self::vec_to_json_ref(v, &info)
        }

        fn vec_to_json_ref(v: Vec<SqliteRow>, info: &Self::RowInfo) -> serde_json::Value {
            let mut out = Vec::new();
            for row in v {
                for (name, nullable, type_name) in info {
                    match (nullable, type_name) {
                        (true, TypeName::Text) => {
                            match row.get::<'_, Option<String>, _>(name.as_str()) {
                                Some(s) => out.push(Value::String(s)),
                                None => out.push(Value::Null),
                            }
                        }
                        (true, TypeName::Integer) => {
                            match row.get::<'_, Option<i64>, _>(name.as_str()) {
                                Some(i) => out.push(Value::Number(i.into())),
                                None => out.push(Value::Null),
                            }
                        }
                        (true, TypeName::Real) => {
                            match row.get::<'_, Option<f64>, _>(name.as_str()) {
                                Some(f) => out
                                    .push(Value::Number(serde_json::Number::from_f64(f).unwrap())),
                                None => out.push(Value::Null),
                            }
                        }
                        (true, TypeName::Numeric) => {
                            match row.get::<'_, Option<f64>, _>(name.as_str()) {
                                Some(f) => out
                                    .push(Value::Number(serde_json::Number::from_f64(f).unwrap())),
                                None => out.push(Value::Null),
                            }
                        }
                        (false, TypeName::Text) => {
                            out.push(Value::String(row.get::<'_, String, _>(name.as_str())));
                        }
                        (false, TypeName::Integer) => {
                            out.push(Value::Number(row.get::<'_, i64, _>(name.as_str()).into()));
                        }
                        (false, TypeName::Real) => {
                            out.push(Value::Number(
                                serde_json::Number::from_f64(row.get::<'_, f64, _>(name.as_str()))
                                    .unwrap(),
                            ));
                        }
                        (false, TypeName::Numeric) => {
                            out.push(Value::Number(
                                serde_json::Number::from_f64(row.get::<'_, f64, _>(name.as_str()))
                                    .unwrap(),
                            ));
                        }
                    }
                }
            }
            Value::Array(out)
        }
        fn to_json(row: &SqliteRow) -> serde_json::Value {
            let mut entry = serde_json::Map::default();
            let cols = row.columns();
            for col in cols {
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
            Value::Object(entry)
        }
    }
}
