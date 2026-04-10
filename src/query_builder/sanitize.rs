use sqlx::Sqlite;

use crate::{Expression, QueryBuilder, SanitzingMechanisim};

pub trait SanitizeAndHardcode<Escape> {
    fn sanitize(&self) -> String;
}

pub struct by_double_quote;

/// explicitly hardcode the inner value
pub struct hardcode<T>(pub T);

impl SanitizeAndHardcode<by_double_quote> for bool {
    fn sanitize(&self) -> String {
        match self {
            true => "true",
            false => "false",
        }
        .to_string()
    }
}

impl SanitizeAndHardcode<by_double_quote> for String {
    fn sanitize(&self) -> String {
        let mut new = String::from('\'');
        for (index, char) in self.chars().enumerate() {
            if char == '\'' {
                new.push('"');
            } else {
                new.push(char);
            }
        }
        new.push('\'');
        new
    }
}

impl SanitizeAndHardcode<by_double_quote> for &'_ str {
    fn sanitize(&self) -> String {
        let mut new = String::from('\'');
        for (index, char) in self.chars().enumerate() {
            if char == '\'' {
                new.push('"');
            } else {
                new.push(char);
            }
        }
        new.push('\'');
        new
    }
}

impl<'q, Q, T> Expression<'q, Q> for hardcode<T>
where
    Q: QueryBuilder + SanitzingMechanisim,
    T: SanitizeAndHardcode<Q::SanitzingMechanisim> + 'q,
{
    fn expression(
        self,
        ctx: &mut Q,
    ) -> impl FnOnce(&mut <Q>::Context) -> String + 'q + use<'q, Q, T>
    where
        Q: QueryBuilder,
    {
        move |_| self.0.sanitize()
    }
}
