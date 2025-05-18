use sqlx::SqlitePool;

// #[standard_collection]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

// #[standard_collection]
pub struct Category {
    pub title: String,
}

// #[standard_collection]
pub struct Tag {
    pub title: String,
}

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
}
