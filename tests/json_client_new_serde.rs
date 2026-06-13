#![allow(unused)]
#![warn(unused_must_use)]
use claw_ql::connect_in_memory::ConnectInMemory;
use serde_json::json;
use sqlx::Sqlite;
use std::sync::Arc;

pub enum SupportedType {
    String,
    Boolean,
}

pub struct SupportedTypeVTable {
    pub type_name: fn() -> &'static str,
}

// To selectively disable rustfmt, use #[rustfmt::skip] on the impl and then explicitly resume inside the function body via comments.
// This resumes formatting in the body of `type_name`.

impl SupportedTypeVTable {
    fn new_as<T>() -> Self
    where
        T: Send + Sync + 'static,
    {
        let type_name = || {
            return std::any::type_name::<T>();
        };

        Self { type_name }
    }
    pub fn new(variants: SupportedType) -> Self {
        match variants {
            SupportedType::String => Self::new_as::<String>(),
            SupportedType::Boolean => Self::new_as::<bool>(),
        }
    }
}

pub struct DynamicCollection {
    name: String,
    fields: Vec<(String, SupportedTypeVTable)>,
}

#[tokio::test]
async fn test_new_json() {
    let pool = Sqlite::connect_in_memory().await;

    let j = json!({
        "name": "todo",
        "fields": [
            {
                "name": "title",
                "type_info": "String",
                "is_optional": false,
            },
        ],
    });

    let j: Arc<str> = j.to_string().into();

    let dc = DynamicCollection {
        name: "todo".to_string(),
        // add done, description
        fields: vec![
            ("title".to_string(), SupportedTypeVTable::new_as::<String>()),
            ("done".to_string(), SupportedTypeVTable::new_as::<bool>()),
            (
                "description".to_string(),
                SupportedTypeVTable::new_as::<Option<String>>(),
            ),
        ],
    };
}
