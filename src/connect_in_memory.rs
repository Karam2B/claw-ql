use sqlx::{Database, Pool};

pub trait ConnectInMemory: Database {
    fn in_memory_connection() -> impl Future<Output = <Self as Database>::Connection> + Send;
    fn in_memory_pool() -> impl Future<Output = Pool<Self>>;
}
