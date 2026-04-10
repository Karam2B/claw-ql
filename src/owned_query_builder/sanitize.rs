use sqlx::Type;

use crate::{DatabaseExt, SqlSanitize};

impl<S> SqlSanitize<S> for &'_ str {
    fn to_sql(&self) -> &str {
        self
    }
    fn safe_to_sql(&self) -> bool {
        false
    }
}

impl<S> SqlSanitize<S> for String {
    fn to_sql(&self) -> &str {
        self.as_str()
    }
    fn safe_to_sql(&self) -> bool {
        false
    }
}

impl<S> SqlSanitize<S> for bool {
    fn to_sql(&self) -> &str {
        match self {
            true => "true",
            false => "false",
        }
    }
    fn safe_to_sql(&self) -> bool {
        true
    }
}

pub struct hardcode<T>(pub T);
impl<S, T> SqlSanitize<S> for hardcode<T>
where
    S: DatabaseExt,
    T: Type<S>,
{
    fn to_sql(&self) -> &str {
        todo!("hvae access to DatabaseExt api")
    }
}
