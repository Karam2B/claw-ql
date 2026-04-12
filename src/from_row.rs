use sqlx::Row;

#[allow(non_camel_case_types)]
pub struct pre_alias<'r, R: Row>(pub &'r R, pub &'static str);

#[allow(non_camel_case_types)]
pub struct post_alias<'r, R: Row>(pub &'r R, pub &'static str);

#[derive(Debug)]
pub enum FromRowError {
    MismatchType,
    NotFoundName(String),
}

impl From<sqlx::Error> for FromRowError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::ColumnNotFound(name) => FromRowError::NotFoundName(name),
            _ => panic!(
                "either: 1. incorrect impl of FromRowAlias, 2. uncatchable error like database disconnection"
            ),
        }
    }
}

pub trait FromRowAlias<'r, R> {
    type FromRowData;
    fn no_alias(&self, row: &'r R) -> Result<Self::FromRowData, FromRowError>;
    fn pre_alias(&self, row: pre_alias<'r, R>) -> Result<Self::FromRowData, FromRowError>
    where
        R: Row;
    fn post_alias(&self, row: post_alias<'r, R>) -> Result<Self::FromRowData, FromRowError>
    where
        R: Row;
    fn row_no_alias(row: &'r R, this: &Self) -> Result<Self::FromRowData, FromRowError> {
        this.no_alias(row)
    }
}

pub mod row_helpers {
    use crate::from_row::FromRowAlias;
    use crate::from_row::FromRowError;
    use crate::from_row::post_alias;
    use crate::from_row::pre_alias;
    use sqlx::Row;

    pub trait AliasRowHelper<'r, Handler>: Row + Sized + 'r {
        type Output;
        fn row_no_alias(&'r self, handler: &Handler) -> Result<Self::Output, FromRowError>;
        fn row_pre_alias(
            &'r self,
            handler: &Handler,
            pre_alias_str: &'static str,
        ) -> Result<Self::Output, FromRowError>;
        fn row_post_alias(
            &'r self,
            handler: &Handler,
            post_alias_str: &'static str,
        ) -> Result<Self::Output, FromRowError>;
    }

    impl<'r, Handler, Row_> AliasRowHelper<'r, Handler> for Row_
    where
        Handler: FromRowAlias<'r, Row_>,
        Row_: Row + Sized + 'r,
    {
        type Output = Handler::FromRowData;
        fn row_no_alias(&'r self, handler: &Handler) -> Result<Handler::FromRowData, FromRowError> {
            handler.no_alias(self)
        }
        fn row_pre_alias(
            &'r self,
            handler: &Handler,
            pre_alias_str: &'static str,
        ) -> Result<Handler::FromRowData, FromRowError> {
            handler.pre_alias(pre_alias(self, pre_alias_str))
        }
        fn row_post_alias(
            &'r self,
            handler: &Handler,
            post_alias_str: &'static str,
        ) -> Result<Handler::FromRowData, FromRowError> {
            handler.post_alias(post_alias(self, post_alias_str))
        }
    }

    pub trait OneRowHelper<'r>: Row + Sized + 'r {
        fn from_row<Data: sqlx::FromRow<'r, Self>>(&'r self) -> Result<Data, FromRowError>;
    }

    impl<'r, Row_> OneRowHelper<'r> for Row_
    where
        Row_: Row + Sized + 'r,
    {
        fn from_row<Data: sqlx::FromRow<'r, Self>>(&'r self) -> Result<Data, FromRowError> {
            Ok(Data::from_row(self)?)
        }
    }

    pub trait ManyRowHelper<'r> {
        type Single: Row + Sized + 'r;
        fn from_rows<Data2: sqlx::FromRow<'r, Self::Single>>(
            &'r self,
        ) -> Result<Vec<Data2>, FromRowError>;
    }

    impl<'r, Row_> ManyRowHelper<'r> for Vec<Row_>
    where
        Row_: Row + Sized + 'r,
    {
        type Single = Row_;
        fn from_rows<Data: sqlx::FromRow<'r, Self::Single>>(
            &'r self,
        ) -> Result<Vec<Data>, FromRowError> {
            let mut r = vec![];
            for each in self {
                r.push(Data::from_row(each)?);
            }
            Ok(r)
        }
    }
}

pub mod collection_to_impl_from_row {
    use sqlx::{FromRow, Row};

    use crate::{
        collections::Collection,
        from_row::{FromRowAlias, FromRowError},
        singlton_default::SingltonDefault,
    };

    pub struct CollectionToImplFromRow<Handler: Collection>(pub Handler::Data);

    impl<'r, R, Handler> FromRow<'r, R> for CollectionToImplFromRow<Handler>
    where
        Handler: Collection,
        R: Row,
        Handler: SingltonDefault + FromRowAlias<'r, R, FromRowData = <Handler as Collection>::Data>,
    {
        fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
            match Handler::singlton_default().no_alias(row) {
                Ok(s) => Ok(CollectionToImplFromRow(s)),
                Err(FromRowError::NotFoundName(name)) => {
                    return Err(sqlx::Error::ColumnNotFound(name));
                }
                Err(_) => panic!("unmatched error"),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use sqlx::Sqlite;

    use crate::{
        connect_in_memory::ConnectInMemory,
        from_row::{FromRowAlias, post_alias, pre_alias},
        test_module::{Category, category},
    };

    #[tokio::test]
    async fn main() {
        let pool = Sqlite::connect_in_memory().await;

        let row = sqlx::query(
            "
        CREATE TABLE Category ( title TEXT );
        INSERT INTO Category (title) VALUES ('cat_1');
        SELECT title as cat_title, title, title as title_ FROM Category;
    ",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let s = category.pre_alias(pre_alias(&row, "cat_")).unwrap();

        assert_eq!(
            s,
            Category {
                title: "cat_1".to_string(),
            },
        );

        let s = category.post_alias(post_alias(&row, "_")).unwrap();

        assert_eq!(
            s,
            Category {
                title: "cat_1".to_string(),
            },
        );

        let s = category.no_alias(&row).unwrap();

        assert_eq!(
            s,
            Category {
                title: "cat_1".to_string(),
            },
        );
    }
}
