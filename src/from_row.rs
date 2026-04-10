use sqlx::Row;

use crate::collections::Collection;

#[allow(non_camel_case_types)]
pub struct pre_alias<'r, R: Row>(pub &'r R, pub &'static str);

#[allow(non_camel_case_types)]
pub struct post_alias<'r, R: Row>(pub &'r R, pub &'static str);

#[derive(Debug)]
pub enum FromRowError {
    MismatchType,
    NotFoundName,
}

impl TryFrom<sqlx::Error> for FromRowError {
    type Error = sqlx::Error;
    fn try_from(value: sqlx::Error) -> Result<Self, Self::Error> {
        match value {
            sqlx::Error::ColumnNotFound(_) => return Ok(Self::NotFoundName),
            sqlx::Error::Decode(_) => return Ok(Self::NotFoundName),
            _ => {}
        };
        Err(value)
    }
}

pub trait FromRowAlias<'r, R>: Collection {
    fn no_alias(&self, row: &'r R) -> Result<Self::Data, FromRowError>;
    fn pre_alias(&self, row: pre_alias<'r, R>) -> Result<Self::Data, FromRowError>
    where
        R: Row;
    fn post_alias(&self, row: post_alias<'r, R>) -> Result<Self::Data, FromRowError>
    where
        R: Row;
}
