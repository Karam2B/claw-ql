pub trait CollectionBasic {
    fn table_name(&self) -> &str;
    fn table_name_lower_case(&self) -> &str;
}

pub trait Collection: CollectionBasic {
    type Partial;
    type Data;
    type Id;
    fn id(&self) -> &Self::Id;
}

pub trait MemberBasic {
    fn name(&self) -> &str;
}

pub trait Member: MemberBasic {
    type Data;
    type Collection;
}

pub trait HasHandler {
    type Handler: Collection;
}

pub trait Id {
    type Data;
    type SqlIdent;
    fn ident(&self) -> Self::SqlIdent;
}

#[derive(Clone)]
pub struct SingleIncremintalInt;

impl Id for SingleIncremintalInt {
    type Data = i64;
    type SqlIdent = &'static str;
    fn ident(&self) -> &'static str {
        "id"
    }
}

mod impl_expression {
    use sqlx::Sqlite;

    use crate::{Expression, OpExpression, collections::SingleIncremintalInt};

    impl OpExpression for SingleIncremintalInt {}

    impl<'q> Expression<'q, Sqlite> for SingleIncremintalInt {
        fn expression(self, ctx: &mut crate::QueryBuilder<'q, Sqlite>)
        // where
        //     S: crate::DatabaseExt,
        {
            ctx.syntax("'id' INT PRIMARY KEY");
        }
    }
}
