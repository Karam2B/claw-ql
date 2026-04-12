use sqlx::Row;

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

pub trait RowToJson {
    type RowInfo;
    fn row_info(&self) -> Self::RowInfo;
    fn vec_to_json_ref(v: Vec<Self>, info: &Self::RowInfo) -> serde_json::Value
    where
        Self: Sized;
    fn vec_to_json(v: Vec<Self>) -> serde_json::Value
    where
        Self: Sized;
    fn to_json(&self) -> serde_json::Value;
}

mod impl_sqlite {
    use super::*;
    use serde_json::Value;
    use serde_json::json;
    use sqlx::Column;
    use sqlx::Row;
    use sqlx::TypeInfo;
    use sqlx::sqlite::SqliteRow;

    impl RowToJson for SqliteRow {
        type RowInfo = Vec<(String, String)>;
        fn row_info(&self) -> Self::RowInfo {
            let cols = self.columns();
            let mut info = Vec::new();
            for col in cols {
                info.push((col.name().to_string(), col.type_info().name().to_string()));
            }
            info
        }
        fn vec_to_json(v: Vec<Self>) -> serde_json::Value
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

        fn vec_to_json_ref(v: Vec<Self>, info: &Self::RowInfo) -> serde_json::Value {
            let mut out = Vec::new();
            for row in v {
                for (name, type_name) in info {
                    match type_name.as_str() {
                        "NULL" => {
                            out.push(json!(null));
                        }
                        "TEXT" => {
                            out.push(Value::String(row.get::<'_, String, _>(name.as_str())));
                        }
                        "INTEGER" => {
                            out.push(Value::Number(row.get::<'_, i64, _>(name.as_str()).into()));
                        }
                        "REAL" | "NUMBERIC" => {
                            out.push(Value::Number(
                                serde_json::Number::from_f64(row.get::<'_, f64, _>(name.as_str()))
                                    .unwrap(),
                            ));
                        }
                        _ => {
                            panic!("{} is not supported", type_name);
                        }
                    }
                }
            }
            Value::Array(out)
        }
        fn to_json(&self) -> serde_json::Value {
            let mut entry = serde_json::Map::default();
            let cols = self.columns();
            for col in cols {
                let name = col.name();
                let s = col.type_info();
                match s.name() {
                    "NULL" => {
                        let _ = entry.insert(name.to_string(), json!(null));
                    }
                    "TEXT" => {
                        entry.insert(name.to_string(), Value::String(self.get(name)));
                    }
                    "INTEGER" => {
                        let v: i64 = self.get(name);
                        entry.insert(name.to_string(), v.into());
                    }
                    "REAL" | "NUMBERIC" => {
                        let v: f64 = self.get(name);
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
