use std::marker::PhantomData;

use claw_ql::{
    migration::{migrate, migrate_relation},
    operations::select_one::{SelectOneOutput, get_one},
};
use claw_ql_macros::{Collection, relation};
use sqlx::SqlitePool;
use tracing::Level;

#[derive(Collection, Debug, PartialEq)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Collection, Debug, PartialEq)]
pub struct Category {
    pub title: String,
}

#[derive(Collection, Debug, PartialEq)]
pub struct Tag {
    pub title: String,
}

relation!(optional_to_many Todo Category);

#[tokio::test]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    migrate::<_, Todo>(&pool).await;
    migrate::<_, Category>(&pool).await;
    migrate::<_, Tag>(&pool).await;
    migrate_relation::<_, Todo, Category>(&pool).await;

    sqlx::query(
        r#"
            INSERT INTO Tag (title) VALUES 
                ('tag_1'), ('tag_2'), ('tag_3');
            INSERT INTO Category (title) VALUES ('category_1'), ('category_2'), ('category_3');

            INSERT INTO Todo (title, done, category_id) VALUES
                ('todo_1', 1, 3),
                ('todo_2', 0, 3),
                ('todo_3', 1, NULL),
                ('todo_4', 0, 1),
                ('todo_5', 1, NULL);
            "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let res = get_one(PhantomData::<Todo>)
        .exec_op(pool.clone())
        .await
        .unwrap();

    pretty_assertions::assert_eq!(
        res,
        SelectOneOutput {
            id: 1,
            attr: Todo {
                title: "todo_1".to_string(),
                done: true,
                description: None
            },
            links: (),
        }
    );
}
