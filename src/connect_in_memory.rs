use sqlx::{Database, Pool};

pub trait ConnectInMemory: Database {
    fn connect_in_memory_2() -> impl Future<Output = <Self as Database>::Connection> + Send;
    fn connect_in_memory() -> impl Future<Output = Pool<Self>>;
}
