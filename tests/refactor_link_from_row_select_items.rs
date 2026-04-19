use claw_ql::{
    extentions::common_expressions::StrAliased,
    from_row::{
        FromRowData, FromRowError,
        row_ext::{FromRowAlias2, RowExt},
    },
};
use sqlx::{ColumnIndex, Decode, Type};

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
    fn str_aliased(&self, alias: &'static str) -> Self::StrAliased {
        format!("{}", alias)
    }
}

impl FromRowData for LinkGenericSelectItems {
    type RData = (i32, String);
}
impl<'r, R> FromRowAlias2<'r, R> for LinkGenericSelectItems
where
    R: RowExt,
    Self: FromRowData<RData = (i32, String)>,
    for<'q> &'q str: ColumnIndex<R::OgRow>,
    i32: Type<R::Database> + for<'r2> Decode<'r2, R::Database>,
    String: Type<R::Database> + for<'r2> Decode<'r2, R::Database>,
{
    fn from_row_alias(&self, row_ext: R) -> Result<Self::RData, FromRowError> {
        let id = row_ext.clone().try_get("id")?;
        let name = row_ext.try_get("name")?;
        Ok((id, name))
    }
}
