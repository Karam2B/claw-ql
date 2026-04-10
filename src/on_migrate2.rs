pub trait OnMigrate {
    type Statements;
    fn statments(&self) -> Self::Statements;
}
