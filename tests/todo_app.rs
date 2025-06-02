use std::marker::PhantomData;

use claw_ql::{
    migration::{migrate, migrate_relation},
    operations::{
        Relation,
        select_one::{SelectOneOutput, get_one},
    },
    schema::Schema,
};
use claw_ql_macros::{Collection, relation};
use sqlx::{Sqlite, SqlitePool};
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

    let schema = Schema::default()
        .infer_db::<Sqlite>()
        .add_collection(PhantomData::<Todo>)
        .add_collection(PhantomData::<Tag>)
        .add_collection(PhantomData::<Category>)
        .add_relation::<Todo, Category>()
        .last_link_is_crud();

    schema.migrate(&pool).await;

    migrate(&pool, PhantomData::<Todo>).await;
    migrate(&pool, PhantomData::<Tag>).await;
    migrate(&pool, PhantomData::<Category>).await;
    migrate_relation(
        &pool,
        Relation {
            from: PhantomData::<Todo>,
            to: PhantomData::<Category>,
        },
    )
    .await;

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
