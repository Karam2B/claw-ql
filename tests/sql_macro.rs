use claw_ql::{
    connect_in_memory::ConnectInMemory,
    links::{Link, relation_optional_to_many::optional_to_many},
};
use claw_ql_macros::sql;
use sqlx::Sqlite;

#[derive(claw_ql_macros::Collection, claw_ql_macros::OnMigrate, claw_ql_macros::FromRowAlias)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(claw_ql_macros::Collection, claw_ql_macros::OnMigrate, claw_ql_macros::FromRowAlias)]
pub struct Category {
    pub title: String,
}

impl Link<todo> for category {
    type Spec = optional_to_many<String, todo, category>;
    fn spec(self, _: &todo) -> Self::Spec {
        optional_to_many {
            foriegn_key: String::from("category_id"),
            from: todo,
            to: self,
        }
    }
}

#[tokio::test]
async fn main() {
    let p = Sqlite::connect_in_memory().await;

    let out = sql!(
        SELECT FROM todo
        LINK category
        WHERE title.eq("first_todo")
        WITH p
    )
    .await;
}
