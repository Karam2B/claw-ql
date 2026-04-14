use sqlx::{ColumnIndex, Decode, Row, Type};

pub mod swich_to_base_id {
    use sqlx::Row;

    use crate::from_row::{post_alias, pre_alias, two_alias};

    pub fn pre_alias_to_base_id<'r, R: Row>(pre_alias: pre_alias<'r, R>) -> pre_alias<'r, R> {
        pre_alias::new(pre_alias.row, "i")
    }
    pub fn two_alias_to_base_id<'r, R: Row>(two_alias: two_alias<'r, R>) -> two_alias<'r, R> {
        two_alias::new(two_alias.row, "i")
    }
    pub fn post_alias_to_base_id<'r, R: Row>(post_alias: post_alias<'r, R>) -> post_alias<'r, R> {
        post_alias::new(post_alias.row, "i")
    }
}

#[allow(non_camel_case_types)]
pub struct pre_alias<'r, R: Row> {
    pub(crate) row: &'r R,
    pub(crate) alias: &'static str,
}

impl<'r, R: Row> Clone for pre_alias<'r, R> {
    fn clone(&self) -> Self {
        Self {
            row: self.row,
            alias: self.alias,
        }
    }
}

impl<'r, R: Row> pre_alias<'r, R> {
    pub fn new(row: &'r R, alias: &'static str) -> Self {
        Self { row, alias }
    }
    pub fn try_get<T>(&self, name: &str) -> Result<T, sqlx::Error>
    where
        T: Type<R::Database> + Decode<'r, R::Database>,
        for<'q> &'q str: ColumnIndex<R>,
    {
        Row::try_get(self.row, format!("{}{}", self.alias, name).as_str())
    }
    #[track_caller]
    pub fn get<T>(&self, name: &str) -> T
    where
        T: Type<R::Database> + Decode<'r, R::Database>,
        for<'q> &'q str: ColumnIndex<R>,
    {
        self.try_get(name).unwrap()
    }
}

#[allow(non_camel_case_types)]
pub struct two_alias<'r, R: Row> {
    pub(crate) row: &'r R,
    pub(crate) str_alias: &'static str,
    // only Vec<T> and tuples, can initiate this with Some(usize)
    pub(crate) num_alias: Option<usize>,
}

impl<'r, R: Row> Clone for two_alias<'r, R> {
    fn clone(&self) -> Self {
        Self {
            row: self.row,
            str_alias: self.str_alias,
            num_alias: self.num_alias,
        }
    }
}

impl<'r, R: Row> two_alias<'r, R> {
    pub fn new(row: &'r R, name: &'static str) -> Self {
        Self {
            row,
            str_alias: name,
            num_alias: None,
        }
    }
    pub fn try_get<T>(&self, name: &str) -> Result<T, sqlx::Error>
    where
        T: Type<R::Database> + Decode<'r, R::Database>,
        for<'q> &'q str: ColumnIndex<R>,
    {
        Row::try_get(
            self.row,
            format!(
                "{}{}{}",
                self.str_alias,
                self.num_alias.map(|e| e.to_string()).unwrap_or_default(),
                name,
            )
            .as_str(),
        )
    }
    #[track_caller]
    pub fn get<T>(&self, name: &str) -> T
    where
        T: Type<R::Database> + Decode<'r, R::Database>,
        for<'q> &'q str: ColumnIndex<R>,
    {
        self.try_get(name).unwrap()
    }
}

#[allow(non_camel_case_types)]
pub struct post_alias<'r, R: Row> {
    pub(crate) row: &'r R,
    pub(crate) alias: &'static str,
}

impl<'r, R: Row> Clone for post_alias<'r, R> {
    fn clone(&self) -> Self {
        Self {
            row: self.row,
            alias: self.alias,
        }
    }
}

impl<'r, R: Row> post_alias<'r, R> {
    pub fn new(row: &'r R, alias: &'static str) -> Self {
        Self { row, alias }
    }
    pub fn try_get<T>(&self, name: &str) -> Result<T, sqlx::Error>
    where
        T: Type<R::Database> + Decode<'r, R::Database>,
        for<'q> &'q str: ColumnIndex<R>,
    {
        Row::try_get(self.row, format!("{}{}", name, self.alias).as_str())
    }
    #[track_caller]
    pub fn get<T>(&self, name: &str) -> T
    where
        T: Type<R::Database> + Decode<'r, R::Database>,
        for<'q> &'q str: ColumnIndex<R>,
    {
        self.try_get(name).unwrap()
    }
}

#[derive(Debug)]
pub enum FromRowError {
    MismatchType,
    ColumnNotFound(String),
}

impl From<sqlx::Error> for FromRowError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::ColumnNotFound(name) => FromRowError::ColumnNotFound(name),
            _ => panic!(
                "{value:?}, either: 1. incorrect impl of FromRowAlias, 2. uncatchable error like database disconnection"
            ),
        }
    }
}

pub trait FromRowData {
    type RData;
}
pub trait FromRowAlias<'r, R>: FromRowData {
    // used in operation with a returning clause
    fn no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError>;
    // used in operation that have local field belong to different links and collections
    fn pre_alias(&self, row: pre_alias<'r, R>) -> Result<Self::RData, FromRowError>
    where
        R: Row;
    // Not used anywhere in my code, I think of deleting this function
    fn post_alias(&self, row: post_alias<'r, R>) -> Result<Self::RData, FromRowError>
    where
        R: Row;
    // used in links, where `Vec<T>` and tuples use an Option<usize>
    fn two_alias(&self, row: two_alias<'r, R>) -> Result<Self::RData, FromRowError>
    where
        R: Row;
}

pub trait TryFromRowAlias<'r, R>: FromRowData {
    fn try_no_alias(&self, row: &'r R) -> Result<Option<Self::RData>, FromRowError>;
    fn try_pre_alias(&self, row: pre_alias<'r, R>) -> Result<Option<Self::RData>, FromRowError>
    where
        R: Row;
    fn try_two_alias(&self, row: two_alias<'r, R>) -> Result<Option<Self::RData>, FromRowError>
    where
        R: Row;
    fn try_post_alias(&self, row: post_alias<'r, R>) -> Result<Option<Self::RData>, FromRowError>
    where
        R: Row;
}

impl FromRowData for () {
    type RData = ();
}

impl<'r, R> FromRowAlias<'r, R> for ()
where
    R: Row,
{
    fn no_alias(&self, _: &'r R) -> Result<Self::RData, FromRowError> {
        Ok(())
    }
    fn pre_alias(&self, _: pre_alias<'r, R>) -> Result<Self::RData, FromRowError> {
        Ok(())
    }
    fn post_alias(&self, _: post_alias<'r, R>) -> Result<Self::RData, FromRowError> {
        Ok(())
    }
    fn two_alias(&self, _: two_alias<'r, R>) -> Result<Self::RData, FromRowError> {
        Ok(())
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
        fn row_two_alias(
            &'r self,
            handler: &Handler,
            pre_alias_str: &'static str,
            pre_alias_num: Option<usize>,
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
        type Output = Handler::RData;
        fn row_no_alias(&'r self, handler: &Handler) -> Result<Handler::RData, FromRowError> {
            handler.no_alias(self)
        }
        fn row_two_alias(
            &'r self,
            handler: &Handler,
            pre_alias_str: &'static str,
            pre_alias_num: Option<usize>,
        ) -> Result<Self::Output, FromRowError> {
            handler.two_alias(super::two_alias {
                row: self,
                str_alias: pre_alias_str,
                num_alias: pre_alias_num,
            })
        }
        fn row_pre_alias(
            &'r self,
            handler: &Handler,
            pre_alias_str: &'static str,
        ) -> Result<Handler::RData, FromRowError> {
            handler.pre_alias(pre_alias::new(self, pre_alias_str))
        }
        fn row_post_alias(
            &'r self,
            handler: &Handler,
            post_alias_str: &'static str,
        ) -> Result<Handler::RData, FromRowError> {
            handler.post_alias(post_alias::new(self, post_alias_str))
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
        singleton::Singleton,
    };

    pub struct CollectionToImplFromRow<Handler: Collection>(pub Handler::Data);

    impl<'r, R, Handler> FromRow<'r, R> for CollectionToImplFromRow<Handler>
    where
        Handler: Collection,
        R: Row,
        Handler: Singleton + FromRowAlias<'r, R, RData = <Handler as Collection>::Data>,
    {
        fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
            match Handler::singleton().no_alias(row) {
                Ok(s) => Ok(CollectionToImplFromRow(s)),
                Err(FromRowError::ColumnNotFound(name)) => {
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

        let s = category.pre_alias(pre_alias::new(&row, "cat_")).unwrap();

        assert_eq!(
            s,
            Category {
                title: "cat_1".to_string(),
            },
        );

        let s = category.post_alias(post_alias::new(&row, "_")).unwrap();

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
