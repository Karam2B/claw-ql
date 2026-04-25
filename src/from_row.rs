use sqlx::{ColumnIndex, Decode, Row, Type};

pub mod swich_to_base_id {
    use sqlx::Row;

    use crate::from_row::{RowPostAliased, RowPreAliased, RowTwoAliased};

    pub fn pre_alias_to_base_id<'r, R: Row>(
        pre_alias: RowPreAliased<'r, R>,
    ) -> RowPreAliased<'r, R> {
        RowPreAliased::new(pre_alias.row, "i")
    }
    pub fn two_alias_to_base_id<'r, R: Row>(
        two_alias: RowTwoAliased<'r, R>,
    ) -> RowTwoAliased<'r, R> {
        RowTwoAliased::new(two_alias.row, "i")
    }
    pub fn post_alias_to_base_id<'r, R: Row>(
        post_alias: RowPostAliased<'r, R>,
    ) -> RowPostAliased<'r, R> {
        RowPostAliased::new(post_alias.row, "i")
    }
}

#[allow(non_camel_case_types)]
pub struct RowPreAliased<'r, R: Row> {
    pub(crate) row: &'r R,
    pub(crate) alias: &'static str,
}

impl<'r, R: Row> Clone for RowPreAliased<'r, R> {
    fn clone(&self) -> Self {
        Self {
            row: self.row,
            alias: self.alias,
        }
    }
}

impl<'r, R: Row> RowPreAliased<'r, R> {
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
pub struct RowTwoAliased<'r, R: Row> {
    pub(crate) row: &'r R,
    pub(crate) str_alias: &'static str,
    // only Vec<T> and tuples, can initiate this with Some(usize)
    pub(crate) num_alias: Option<usize>,
}

impl<'r, R: Row> Clone for RowTwoAliased<'r, R> {
    fn clone(&self) -> Self {
        Self {
            row: self.row,
            str_alias: self.str_alias,
            num_alias: self.num_alias,
        }
    }
}

impl<'r, R: Row> RowTwoAliased<'r, R> {
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
pub struct RowPostAliased<'r, R: Row> {
    pub(crate) row: &'r R,
    pub(crate) alias: &'static str,
}

impl<'r, R: Row> Clone for RowPostAliased<'r, R> {
    fn clone(&self) -> Self {
        Self {
            row: self.row,
            alias: self.alias,
        }
    }
}

impl<'r, R: Row> RowPostAliased<'r, R> {
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

pub mod from_row_v2 {
    use core::fmt;

    use crate::from_row::{
        FromRowData, FromRowError, RowPostAliased, RowPreAliased, RowTwoAliased,
    };
    use sqlx::{ColumnIndex, Database, Decode, Row, Sqlite, Type, sqlite::SqliteRow};
    use std::any::Any;

    pub trait RowAliased: Clone + Sized {
        type SqlxRow: Row;
        type Database: Database;

        fn get_sqlx_row(&self) -> &Self::SqlxRow;

        #[inline]
        #[track_caller]
        fn get<I, T>(self, index: I) -> T
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            T: Type<Self::Database> + for<'r> Decode<'r, Self::Database>,
        {
            self.try_get::<I, T>(index).unwrap()
        }

        fn try_get_optional<I, T>(self, index: I) -> Result<Option<T>, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            Option<T>: Type<Self::Database> + for<'r> Decode<'r, Self::Database>;

        fn try_get<I, T>(self, index: I) -> Result<T, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            T: Type<Self::Database> + for<'r> Decode<'r, Self::Database>;
    }

    impl<'r, R: Row> RowAliased for &'r R
    where
        for<'q> &'q str: ColumnIndex<R>,
    {
        type SqlxRow = R;
        type Database = R::Database;

        fn get_sqlx_row(&self) -> &Self::SqlxRow {
            *self
        }

        fn try_get<I, T>(self, index: I) -> Result<T, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            T: Type<Self::Database> + for<'s> Decode<'s, Self::Database>,
        {
            sqlx::Row::try_get(self.get_sqlx_row(), index)
        }

        fn try_get_optional<I, T>(self, index: I) -> Result<Option<T>, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            Option<T>: Type<Self::Database> + for<'s> Decode<'s, Self::Database>,
        {
            sqlx::Row::try_get(self.get_sqlx_row(), index)
        }
    }
    impl<'r, R: Row> RowAliased for RowPreAliased<'r, R>
    where
        for<'q> &'q str: ColumnIndex<R>,
    {
        type SqlxRow = R;

        type Database = R::Database;

        fn get_sqlx_row(&self) -> &Self::SqlxRow {
            self.row
        }

        fn try_get<I, T>(self, index: I) -> Result<T, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            T: Type<Self::Database> + for<'r2> Decode<'r2, Self::Database>,
        {
            sqlx::Row::try_get(
                self.row,
                format!("{}{}", self.alias, index.to_string()).as_str(),
            )
        }
        fn try_get_optional<I, T>(self, index: I) -> Result<Option<T>, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            Option<T>: Type<Self::Database> + for<'r2> Decode<'r2, Self::Database>,
        {
            sqlx::Row::try_get(
                self.row,
                format!("{}{}", self.alias, index.to_string()).as_str(),
            )
        }
    }

    impl<'r, R: Row> RowAliased for RowTwoAliased<'r, R>
    where
        for<'q> &'q str: ColumnIndex<R>,
    {
        type SqlxRow = R;

        type Database = R::Database;

        fn get_sqlx_row(&self) -> &Self::SqlxRow {
            self.row
        }

        fn try_get<I, T>(self, index: I) -> Result<T, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            T: Type<Self::Database> + for<'r2> Decode<'r2, Self::Database>,
        {
            sqlx::Row::try_get(
                self.row,
                format!(
                    "{}{}{}",
                    self.str_alias,
                    self.num_alias.unwrap_or_default(),
                    index.to_string()
                )
                .as_str(),
            )
        }
        fn try_get_optional<I, T>(self, index: I) -> Result<Option<T>, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            Option<T>: Type<Self::Database> + for<'r2> Decode<'r2, Self::Database>,
        {
            sqlx::Row::try_get(
                self.row,
                format!(
                    "{}{}{}",
                    self.str_alias,
                    self.num_alias.unwrap_or_default(),
                    index.to_string()
                )
                .as_str(),
            )
        }
    }

    impl<'r, R: Row> RowAliased for RowPostAliased<'r, R>
    where
        for<'q> &'q str: ColumnIndex<R>,
    {
        type SqlxRow = R;

        type Database = R::Database;

        fn get_sqlx_row(&self) -> &Self::SqlxRow {
            self.row
        }

        fn try_get<I, T>(self, index: I) -> Result<T, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            T: Type<Self::Database> + for<'r2> Decode<'r2, Self::Database>,
        {
            sqlx::Row::try_get(
                self.row,
                format!("{}{}", index.to_string(), self.alias).as_str(),
            )
        }
        fn try_get_optional<I, T>(self, index: I) -> Result<Option<T>, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            Option<T>: Type<Self::Database> + for<'r2> Decode<'r2, Self::Database>,
        {
            sqlx::Row::try_get(
                self.row,
                format!("{}{}", index.to_string(), self.alias).as_str(),
            )
        }
    }

    pub trait FromRowAlias2<'r, RAlias>: FromRowData {
        fn from_row_alias(&self, row_aliased: RAlias) -> Result<Self::RData, FromRowError>;
    }
}

pub trait FromRowData {
    type RData;
}
pub trait FromRowAlias<'r, R>: FromRowData {
    // used in operation with a returning clause
    fn no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError>;
    // used in operation that have local field belong to different links and collections
    fn pre_alias(&self, row: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError>
    where
        R: Row;
    // Not used anywhere in my code, I think of deleting this function
    fn post_alias(&self, row: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError>
    where
        R: Row;
    // used in links, where `Vec<T>` and tuples use an Option<usize>
    fn two_alias(&self, row: RowTwoAliased<'r, R>) -> Result<Self::RData, FromRowError>
    where
        R: Row;
}

#[claw_ql_macros::skip]
mod functional_impls {
    use crate::from_row::FromRowAlias;
    use crate::from_row::FromRowData;
    use crate::from_row::FromRowError;
    use crate::from_row::RowPostAliased;
    use crate::from_row::RowPreAliased;
    use crate::from_row::RowTwoAliased;
    use sqlx::Row;

    impl<T> FromRowData for Vec<T>
    where
        T: FromRowData,
    {
        type RData = Vec<T::RData>;
    }
    impl<'r, T, R> FromRowAlias<'r, R> for Vec<T>
    where
        T: FromRowAlias<'r, R>,
    {
        fn no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError> {
            let mut r = vec![];
            for each in self {
                r.push(each.no_alias(row)?);
            }
            Ok(r)
        }

        fn pre_alias(&self, row: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError>
        where
            R: Row,
        {
            let mut r = vec![];
            for each in self {
                r.push(each.pre_alias(row)?);
            }
            Ok(r)
        }

        fn post_alias(&self, row: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError>
        where
            R: Row,
        {
            let mut r = vec![];
            for each in self {
                r.push(each.post_alias(row)?);
            }
            Ok(r)
        }

        fn two_alias(&self, row: RowTwoAliased<'r, R>) -> Result<Self::RData, FromRowError>
        where
            R: Row,
        {
            let mut r = vec![];
            for each in self {
                r.push(each.two_alias(row)?);
            }
            Ok(r)
        }
    }
}

pub trait TryFromRowAlias<'r, R>: FromRowData {
    fn try_no_alias(&self, row: &'r R) -> Result<Option<Self::RData>, FromRowError>;
    fn try_pre_alias(&self, row: RowPreAliased<'r, R>) -> Result<Option<Self::RData>, FromRowError>
    where
        R: Row;
    fn try_two_alias(&self, row: RowTwoAliased<'r, R>) -> Result<Option<Self::RData>, FromRowError>
    where
        R: Row;
    fn try_post_alias(
        &self,
        row: RowPostAliased<'r, R>,
    ) -> Result<Option<Self::RData>, FromRowError>
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
    fn pre_alias(&self, _: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError> {
        Ok(())
    }
    fn post_alias(&self, _: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError> {
        Ok(())
    }
    fn two_alias(&self, _: RowTwoAliased<'r, R>) -> Result<Self::RData, FromRowError> {
        Ok(())
    }
}

pub mod row_helpers {
    use crate::from_row::FromRowAlias;
    use crate::from_row::FromRowError;
    use crate::from_row::RowPostAliased;
    use crate::from_row::RowPreAliased;
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
            handler.two_alias(super::RowTwoAliased {
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
            handler.pre_alias(RowPreAliased::new(self, pre_alias_str))
        }
        fn row_post_alias(
            &'r self,
            handler: &Handler,
            post_alias_str: &'static str,
        ) -> Result<Handler::RData, FromRowError> {
            handler.post_alias(RowPostAliased::new(self, post_alias_str))
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
        from_row::{FromRowAlias, RowPostAliased, RowPreAliased},
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

        let s = category
            .pre_alias(RowPreAliased::new(&row, "cat_"))
            .unwrap();

        assert_eq!(
            s,
            Category {
                title: "cat_1".to_string(),
            },
        );

        let s = category.post_alias(RowPostAliased::new(&row, "_")).unwrap();

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
