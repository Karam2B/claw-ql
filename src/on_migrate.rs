trait OnMigrate<S> {
    type Statement;
    fn migrate(&self) -> Self::Statement;
}

trait LiqOnMigrate<S: Database> {
    fn migrate(&self, pool: Pool<S>) -> Box<dyn Future<Output = ()>>;
}

impl<T, S> LiqOnMigrate<S> for T
where
    S: Database + QueryBuilder,
    T: OnMigrate2<S>,
    T::Statement: Buildable<Database = S>,
    T::Statement: StatementMayNeverBind,
{
    fn migrate(&self, pool: Pool<S>) -> Box<dyn Future<Output = ()>> {
        let str = OnMigrate2::migrate(self).build();
        todo!()
    }
}

impl OnMigrate2<Sqlite> for SingleIncremintalInt {
    type Statement = CreateTableSt<Sqlite>;
    fn migrate(&self) -> Self::Statement {
        let mut stmt = CreateTableSt::init(header::create, <Self as Id>::ident());
        stmt.column_def("id", crate::expressions::exports::primary_key::<Sqlite>());
        stmt
    }
}

impl<F> OnMigrate2<Sqlite> for date_spec<F> {
    type Statement = (AlterTableAddColumn, AlterTableAddColumn, CreateTrigger);
    fn migrate(&self) -> Self::Statement {
        todo!()
    }
}

impl OnMigrate<Sqlite> for SingleIncremintalInt {
    fn custom_migrate_statements(&self) -> Vec<String> {
        let mut stmt = CreateTableSt::init(header::create, <Self as Id>::ident());
        stmt.column_def("id", crate::expressions::exports::primary_key::<Sqlite>());
        vec![Buildable::build(stmt).0]
    }
}
