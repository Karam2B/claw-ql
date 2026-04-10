impl ConnectInMemory for sqlx::Any {
    #[track_caller]
    fn connect_in_memory() -> impl Future<Output = Pool<Self>> {
        if Self::Name != "sqlite" {
            panic!("you cannot connect in memory for other thatn sqlite")
        }
        async { AnyPool::connect("sqlite::memory:").await.unwrap() }
    }
}
