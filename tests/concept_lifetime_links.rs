use sqlx::sqlite::SqliteRow;

pub trait LinkLifetime {
    type SubOp<'r>;
    fn sub_op<'r>(&self, row: &'r SqliteRow) -> Self::SubOp<'r>;
}
