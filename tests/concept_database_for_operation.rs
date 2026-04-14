use sqlx::{ColumnIndex, Database, Decode, Encode, Row, Sqlite, Type};

pub trait ToBind {}

pub trait DatabaseForOperation: Database {
    type EncodedI64: for<'q> Encode<'q, Self> + Type<Self>;
    fn encode_i64(&self, value: i64) -> Self::EncodedI64;
    fn decode_i64_str(&self, row: &Self::Row, key: &str) -> i64;
}

impl<S> DatabaseForOperation for S
where
    S: Database,
    i64: for<'q> Encode<'q, S> + Type<S>,
    i64: for<'q> Decode<'q, S>,
    for<'q> &'q str: ColumnIndex<S::Row>,
{
    type EncodedI64 = i64;
    #[inline]
    fn encode_i64(&self, value: i64) -> Self::EncodedI64 {
        value
    }

    fn decode_i64_str(&self, row: &Self::Row, key: &str) -> i64 {
        Row::get(row, key)
    }
}

// pub struct DynEncodeI64;

// impl Type<Box<dyn DynamicDatabase + Send>> for DynEncodeI64 {
//     fn type_info() -> <Box<dyn DynamicDatabase + Send> as Database>::TypeInfo {
//         todo!()
//     }
// }

// impl Database for Box<dyn DynamicDatabase + Send> {
//     type Connection = ();

//     type TransactionManager;

//     type Row;

//     type QueryResult;

//     type Column;

//     type TypeInfo;

//     type Value;

//     type ValueRef<'r>;

//     type Arguments<'q>;

//     type ArgumentBuffer<'q>;

//     type Statement<'q>;

//     const NAME: &'static str;

//     const URL_SCHEMES: &'static [&'static str];
// }

/* full circle impl is kinda of alot of work */
pub trait DynamicDatabase {
    fn encode_i64(&self, value: i64) -> Box<dyn ToBind>;
}

impl<T> DynamicDatabase for T
where
    T: DatabaseForOperation,
{
    fn encode_i64(&self, value: i64) -> Box<dyn ToBind> {
        todo!()
    }
}

/* full circle impl */
// impl DatabaseForOperation for Box<dyn DynamicDatabase + Send> {
//     type EncodedI64 = DynEncodeI64;

//     fn encode_i64(&self, value: i64) -> Self::EncodedI64 {
//         todo!()
//     }

//     fn decode_i64_str(&self, row: &Self::Row, key: &str) -> i64 {
//         todo!()
//     }
// }

fn assert<S>(_: S) -> Result<(), String> {
    Ok(())
}

#[test]
fn tests() {
    assert(Sqlite).unwrap();

    // check if dyn-compatiable
    let _: Box<dyn DynamicDatabase> = (|| todo!())();

    let _ = Box::new(Sqlite) as Box<dyn DynamicDatabase>;
}
