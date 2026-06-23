use crate::{
    database_extention::DatabaseExt,
    expressions::{
        ColumnEqual,
        filters::{
            ColumnContains, ColumnGreaterThan, ColumnGreaterThanOrEqual, ColumnIsNotNull,
            ColumnIsNull, ColumnLessThan, ColumnLessThanOrEqual, ColumnNotEqual, FilterAnd,
        },
    },
    gen_serde::json_format_side::PartialDeserialize,
    json_client::{
        ToBind, client_interface::SupportedFilter, dynamic_collection::DynamicCollection,
    },
    sqlx_query_builder::{
        basic_expressions::{ExpressionsWithAnd, ExpressionsWithOr},
        trait_objects::BoxedExpression,
    },
    sub_arc::ArcSubStr,
};

fn field_by_col<'a, S>(
    col: &ArcSubStr,
    base: &'a DynamicCollection<S>,
) -> Result<&'a crate::json_client::dynamic_collection::DynamicField<S>, ()>
where
    S: DatabaseExt,
{
    base.fields
        .iter()
        .find(|f| f.name.as_str() == col.as_str())
        .ok_or(())
}

fn field_bind<S>(
    col: ArcSubStr,
    partial: PartialDeserialize,
    base: &DynamicCollection<S>,
) -> Result<(ArcSubStr, Box<dyn ToBind<S> + Send>), ()>
where
    S: DatabaseExt,
{
    let field = field_by_col(&col, base)?;
    let bind = (field.type_info.to_bind)(partial)?;
    Ok((col, bind))
}

fn string_contains_bind<S>(
    col: ArcSubStr,
    partial: PartialDeserialize,
    base: &DynamicCollection<S>,
) -> Result<(ArcSubStr, Box<dyn ToBind<S> + Send>), ()>
where
    S: DatabaseExt,
    String: for<'q> sqlx::Encode<'q, S> + sqlx::Type<S>,
{
    let field = field_by_col(&col, base)?;
    if (field.type_info.type_name)() != std::any::type_name::<String>() {
        return Err(());
    }
    let needle: String = partial.continue_deserialize().map_err(|_| ())?;
    let pattern = format!("%{}%", needle);
    Ok((col, Box::new(pattern)))
}

pub fn parse_one_supported_filter<'q, S>(
    filter: SupportedFilter,
    base: &DynamicCollection<S>,
) -> Result<Box<dyn BoxedExpression<S> + Send>, ()>
where
    S: DatabaseExt,
    ColumnEqual<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnNotEqual<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnGreaterThan<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnGreaterThanOrEqual<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnLessThan<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnLessThanOrEqual<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnContains<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnIsNull<ArcSubStr>: BoxedExpression<S>,
    ColumnIsNotNull<ArcSubStr>: BoxedExpression<S>,
    String: for<'a> sqlx::Encode<'a, S> + sqlx::Type<S>,
{
    Ok(match filter {
        SupportedFilter::ColEq(ColumnEqual { col, eq }) => {
            let (col, bind) = field_bind(col, eq, base)?;
            Box::new(ColumnEqual { col, eq: bind })
        }
        SupportedFilter::ColNe { col, ne } => {
            let (col, bind) = field_bind(col, ne, base)?;
            Box::new(ColumnNotEqual { col, ne: bind })
        }
        SupportedFilter::ColGt { col, gt } => {
            let (col, bind) = field_bind(col, gt, base)?;
            Box::new(ColumnGreaterThan { col, val: bind })
        }
        SupportedFilter::ColGte { col, gte } => {
            let (col, bind) = field_bind(col, gte, base)?;
            Box::new(ColumnGreaterThanOrEqual { col, val: bind })
        }
        SupportedFilter::ColLt { col, lt } => {
            let (col, bind) = field_bind(col, lt, base)?;
            Box::new(ColumnLessThan { col, val: bind })
        }
        SupportedFilter::ColLte { col, lte } => {
            let (col, bind) = field_bind(col, lte, base)?;
            Box::new(ColumnLessThanOrEqual { col, val: bind })
        }
        SupportedFilter::ColContains { col, value } => {
            let (col, bind) = string_contains_bind(col, value, base)?;
            Box::new(ColumnContains { col, val: bind })
        }
        SupportedFilter::ColIsNull { col } => {
            field_by_col(&col, base)?;
            Box::new(ColumnIsNull { col })
        }
        SupportedFilter::ColIsNotNull { col } => {
            field_by_col(&col, base)?;
            Box::new(ColumnIsNotNull { col })
        }
        SupportedFilter::And { filters } => {
            let inner = parse_supported_filter(filters, base)?;
            Box::new(ExpressionsWithAnd(inner))
        }
        SupportedFilter::Or { filters } => {
            let inner = parse_supported_filter(filters, base)?;
            Box::new(ExpressionsWithOr(inner))
        }
    })
}

pub fn parse_supported_filter<'q, S>(
    input: Vec<SupportedFilter>,
    base: &DynamicCollection<S>,
) -> Result<Vec<Box<dyn BoxedExpression<S> + Send>>, ()>
where
    S: DatabaseExt,
    ColumnEqual<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnNotEqual<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnGreaterThan<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnGreaterThanOrEqual<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnLessThan<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnLessThanOrEqual<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnContains<ArcSubStr, Box<dyn ToBind<S> + Send>>: BoxedExpression<S>,
    ColumnIsNull<ArcSubStr>: BoxedExpression<S>,
    ColumnIsNotNull<ArcSubStr>: BoxedExpression<S>,
    String: for<'a> sqlx::Encode<'a, S> + sqlx::Type<S>,
{
    input
        .into_iter()
        .map(|filter| parse_one_supported_filter(filter, base))
        .collect()
}
