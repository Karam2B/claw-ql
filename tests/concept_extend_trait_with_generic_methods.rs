#![allow(unused)]
use core::fmt;
use std::any::Any;

use claw_ql::{
    connect_in_memory::ConnectInMemory,
    database_extention::DatabaseExt,
    extentions::common_expressions::StrAliased,
    from_row::{
        FromRowData, FromRowError, RowPreAliased,
        from_row_v2::{FromRowAlias2, RowAliased},
    },
    operations::{Operation, OperationOutput},
    query_builder::{ManyBoxedExpressions, ManyExpressions},
};
use sqlx::{ColumnIndex, Database, Decode, IntoArguments, Sqlite, Type};

pub trait Link {
    type SelectItems;
    fn select_items(&self) -> Self::SelectItems;
}

pub struct LinkGeneric;
pub struct LinkGenericSelectItems;

impl Link for LinkGeneric {
    type SelectItems = LinkGenericSelectItems;
    fn select_items(&self) -> Self::SelectItems {
        LinkGenericSelectItems
    }
}

impl StrAliased for LinkGenericSelectItems {
    type StrAliased = String;
    fn str_aliased(&self, _: &'static str) -> Self::StrAliased {
        format!("2 as id, 'name' as name")
    }
}

impl FromRowData for LinkGenericSelectItems {
    type RData = (i32, String);
}
impl<'r, R> FromRowAlias2<'r, R> for LinkGenericSelectItems
where
    R: RowAliased,
    Self: FromRowData<RData = (i32, String)>,
    for<'q> &'q str: ColumnIndex<R::SqlxRow>,
    i32: Type<R::Database> + for<'r2> Decode<'r2, R::Database>,
    String: Type<R::Database> + for<'r2> Decode<'r2, R::Database>,
{
    fn from_row_alias(&self, row_ext: R) -> Result<Self::RData, FromRowError> {
        let id = row_ext.clone().try_get("id")?;
        let name = row_ext.try_get("name")?;
        Ok((id, name))
    }
}

pub struct OperationExample<T>(pub T);

impl<T> OperationOutput for OperationExample<T>
where
    T: Link<SelectItems: FromRowData>,
{
    type Output = <T::SelectItems as FromRowData>::RData;
}
impl<S, T> Operation<S> for OperationExample<T>
where
    S: DatabaseExt,
    T: Link + Send,
    T::SelectItems: StrAliased<StrAliased: fmt::Display>,
    T::SelectItems: Send + FromRowData<RData: Send>,
    T::SelectItems: for<'r> FromRowAlias2<'r, <S as Database>::Row>,
    for<'m> &'m mut <S as sqlx::Database>::Connection: sqlx::Executor<'m, Database = S>,
    for<'a> <S as sqlx::Database>::Arguments<'a>: IntoArguments<'a, S>,
{
    fn exec_operation(self, pool: &mut S::Connection) -> impl Future<Output = Self::Output> + Send
    where
        S: Database,
        Self: Sized,
    {
        async move {
            let row = sqlx::query(
                format!(
                    "
                SELECT {}
            ",
                    self.0.select_items().str_aliased("irrelavant")
                )
                .as_str(),
            )
            .fetch_one(pool)
            .await
            .unwrap();

            let data = self.0.select_items().from_row_alias(row).unwrap();

            data
        }
    }
}

pub struct SelectItemsVtable<'r, S: Database, FromRowResult> {
    pub many_expressions: fn(&'static str) -> Box<dyn ManyBoxedExpressions<S> + Send>,
    pub from_ref_row: fn(&'r S::Row) -> FromRowResult,
}

pub trait DynamicLink {}

impl<T> DynamicLink for T {}

impl Link for Box<dyn DynamicLink + Send> {
    type SelectItems = ();
    fn select_items(&self) -> Self::SelectItems {
        todo!()
    }
}

// pub mod v2 {
//     use claw_ql::database_extention::DatabaseExt;
//     use claw_ql::from_row::FromRowData;
//     use core::fmt;
//     use std::any::Any;

//     pub struct DynamicSelectItems<BaseLink, S: DatabaseExt> {
//         pub base_link: BaseLink,
//         pub select_items: Box<dyn Any + Send>,
//     }

//     impl<BaseLink, S: DatabaseExt> FromRowData for DynamicSelectItems<BaseLink, S> {
//         type RData = Box<dyn fmt::Debug + Send>;
//     }
// }
mod v1 {
    use claw_ql::database_extention::DatabaseExt;
    use claw_ql::extentions::common_expressions::StrAliased;
    use claw_ql::from_row::FromRowError;
    use claw_ql::from_row::RowPreAliased;
    use claw_ql::from_row::from_row_v2::FromRowAlias2;
    use claw_ql::from_row::{FromRowAlias, FromRowData};
    use claw_ql::query_builder::ManyExpressions;
    use core::fmt;
    use sqlx::ColumnIndex;
    use sqlx::Database;
    use sqlx::Decode;
    use sqlx::Type;
    use std::any::Any;

    pub trait DynamicSelectItems<S: DatabaseExt> {
        // fn select_items(&self, alias: &'static str) -> Box<dyn ManyBoxedExpressions<S> + Send>;
        // fn from_row(&self, row: &<S as Database>::Row) -> Result<Box<dyn Any + Send>, FromRowError>;
        // fn from_row_alias(
        //     &self,
        //     row: RowPreAliased<'_, <S as Database>::Row>,
        // ) -> Result<Box<dyn Any + Send>, FromRowError>;
    }

    impl<T, S: DatabaseExt> DynamicSelectItems<S> for T
    where
        T: for<'r> FromRowAlias2<'r, &'r <S as Database>::Row>,
        T: for<'r> FromRowAlias2<'r, RowPreAliased<'r, <S as Database>::Row>>,
        T: FromRowData<RData: Any + Send>,
        T: StrAliased<StrAliased: Send + for<'q> ManyExpressions<'q, S>>,
    {
        // fn select_items(&self, alias: &'static str) -> Box<dyn ManyBoxedExpressions<S> + Send> {
        //     Box::new(self.str_aliased(alias))
        // }

        // fn from_row(&self, row: &<S as Database>::Row) -> Result<Box<dyn Any + Send>, FromRowError> {
        //     FromRowAlias2::from_row_alias(self, row).map(|e| Box::new(e) as Box<dyn Any + Send>)
        // }

        // fn from_row_alias(
        //     &self,
        //     row: RowPreAliased<'_, <S as Database>::Row>,
        // ) -> Result<Box<dyn Any + Send>, FromRowError> {
        //     FromRowAlias2::from_row_alias(self, row).map(|e| Box::new(e) as Box<dyn Any + Send>)
        // }
    }

    impl FromRowData for Box<dyn DynamicSelectItems<S>> {
        type RData = Box<dyn fmt::Debug>;
    }

    impl<'r, S> FromRowAlias2<'r, &'r <S as Database>::Row> for Box<dyn DynamicSelectItems<S>>
    where
        S: DatabaseExt,
        Self: FromRowData<RData = (i32, String)>,
        for<'q> &'q str: ColumnIndex<<S as Database>::Row>,
        i32: Type<S> + for<'r2> Decode<'r2, S>,
        String: Type<S> + for<'r2> Decode<'r2, S>,
    {
        fn from_row_alias(
            &self,
            row_ext: &'r <S as Database>::Row,
        ) -> Result<Self::RData, FromRowError> {
            todo!()
        }
    }
}

#[tokio::test]
async fn client() {
    let mut pool = Sqlite::connect_in_memory_2().await;

    let select_items = Box::new(LinkGenericSelectItems) as Box<dyn DynamicLink<Sqlite>>;

    let data = Operation::<Sqlite>::exec_operation(OperationExample(select_items), &mut pool).await;

    assert_eq!(data, (2, "name".to_string()));
}

#[claw_ql_macros::skip]
// failed attempt because sqlx::Row::get is generic over `T` and it is returned, and unboxable
mod boxed_row_aliased {
    use core::fmt;
    use sqlx::{ColumnIndex, Database, Decode, Row, Type};
    use std::any::Any;

    use crate::from_row::from_row_v2::RowAliased;

    pub trait BoxedRowAliased<'r, S> {
        fn try_get(self: Box<Self>, index: String) -> Result<Box<dyn Any + Send>, sqlx::Error>;
        fn try_get_optional(
            self: Box<Self>,
            index: String,
        ) -> Result<Option<Box<dyn Any + Send>>, sqlx::Error>;
    }

    impl<'r, RAlias, S> BoxedRowAliased<'r, S> for RAlias
    where
        S: Database,
        RAlias: RowAliased<Database = S>,
        String: Type<RAlias::Database> + for<'r2> Decode<'r2, RAlias::Database>,
        for<'q> &'q str: ColumnIndex<RAlias::SqlxRow>,
    {
        fn try_get(self: Box<Self>, index: String) -> Result<Box<dyn Any + Send>, sqlx::Error> {
            let _: String = RowAliased::try_get(*self, index.as_str())?;

            todo!("I cannot add `T` to the line above")
        }
        fn try_get_optional(
            self: Box<Self>,
            index: String,
        ) -> Result<Option<Box<dyn Any + Send>>, sqlx::Error> {
            let _ = index;
            todo!()
        }
    }

    pub struct ClonableBox<T: ?Sized>(pub Box<T>);
    impl<T: ?Sized> Clone for ClonableBox<T> {
        fn clone(&self) -> Self {
            todo!()
        }
    }

    impl<'r, S: Database> RowAliased for ClonableBox<dyn BoxedRowAliased<'r, S> + Send> {
        type SqlxRow = S::Row;

        type Database = S;

        fn get_sqlx_row(&self) -> &Self::SqlxRow {
            todo!()
        }

        fn try_get_optional<I, T>(self, index: I) -> Result<Option<T>, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            Option<T>: Type<Self::Database> + for<'r2> Decode<'r2, Self::Database>,
        {
            let index = index.to_string();
            let g: Box<dyn Any + Send> = BoxedRowAliased::try_get(self.0, index)?;
            // self.0.try_get(index)

            todo!()
        }

        fn try_get<I, T>(self, index: I) -> Result<T, sqlx::Error>
        where
            I: ColumnIndex<Self::SqlxRow> + fmt::Display,
            T: Type<Self::Database> + for<'r> Decode<'r, Self::Database>,
        {
            todo!()
        }
    }
}
