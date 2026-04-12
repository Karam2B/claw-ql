use sqlx::Database;

pub trait DatabaseExt: Database {
    fn sanitize(string: &str, into: &mut String);
}
