#![deny(unused_must_use)]

use sqlx::Database;

pub trait DatabaseExt: Database {
    fn sanitize_start(into: &mut String);
    fn sanitize_end(into: &mut String);
    fn sanitize(string: &str, into: &mut String);
}
