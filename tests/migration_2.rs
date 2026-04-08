#![allow(unused)]
#![deny(unused_must_use)]
use claw_ql::ConnectInMemory;
use claw_ql_macros::{Collection, relation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Sqlite;

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
}

#[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub title: String,
}

pub struct TagTable;

#[allow(non_upper_case_globals, dead_code)]
impl TagTable {
    const title: TagTitle = TagTitle;
}
pub struct TagTitle;

pub struct CategoryTable;
pub struct CategoryTitle;

pub struct TodoTable;
pub struct TodoTitle;
pub struct TodoDone;
pub struct TodoDescription;

// struct Id {
//     src: String,
//     id: String,
// }

// trait UniqueId<Registery> {}

fn mm() {
    let _s = json!({
        "resources": [
            { "id": 1, "version": [ { "03/08/2025": { "name": "Todo" } } ] },
            { "id": 2, "version": [ { "03/08/2025": { "name": "Todo.title", "type": "string" } } ] },
            { "id": 2, "version": "03/08/2025" },
            { "id": 3, "version": "03/08/2025" },
            { "id": 4, },
            { "id": 5, },
            { "id": 6, },
            { "id": 7, },
            { "id": 8, },
        ],
        "dep_graph": {
            "3": [2, 3, 4],
            "5": [6],
            "7": [8],
        },
    });
}

relation!(optional_to_many Todo Category);
relation!(many_to_many Todo Tag);

#[allow(non_camel_case_types, non_upper_case_globals, dead_code)]
mod dd {
    use crate::todo;

    pub struct todo_title;
    pub struct todo_done;
    pub struct todo_description;
    impl todo {
        pub const title: todo_title = todo_title;
        pub const done: todo_done = todo_done;
        pub const description: todo_description = todo_description;
    }
}

#[tokio::test]
async fn test_migrate() {
    let schema = Schema {
        collections: (todo, todo::description, category, todo),
        relations: ((todo, category), (todo, tag)),
        links: (set_new, set_id),
    };

    let sql = Sqlite::connect_in_memory().await;

    // let s = sqlx::query("connect").execute(&sql).await.unwrap();

    // migrate(&schema)
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct set_id;

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct set_new;

#[derive(Clone)]
pub struct Schema<C, R, L> {
    pub collections: C,
    pub relations: R,
    pub links: L,
}
