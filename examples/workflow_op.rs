use std::marker::PhantomData;

use claw_ql::select_one::get_one;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct Todo {
    // #[field(Regex(r#"^.{1,100}$"#))]
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[tokio::main]
async fn main() {
    let _db = Pool::<Sqlite>::connect("").await.unwrap();

    // let res = get_one(PhantomData::<Todo>)
    //     // .::<Category, _, _>(|r| {
    //     //     // in theory you can do multiple deep relations and/or go deeper
    //     //     // for now this is only supported for one-time of
    //     //     // C1 --optional_to_many--> C2 --optional_to_many_inverse--> C1
    //     //     r.deep_populate::<Todo>()
    //     // })
    //     .exec_op(db.clone())
    //     .await;
    //
    // pretty_assertions::assert_eq!(
    //     res,
    //     Some(GetOneOutput {
    //         id: 1,
    //         attr: Todo {
    //             title: "todo_1".to_string(),
    //             done: true,
    //             description: None,
    //         },
    //         links: TupleAsMap((Some(GetOneOutput {
    //             id: 3,
    //             attr: Category {
    //                 cat_title: "category_3".to_string()
    //             },
    //             links: (vec![
    //                 SimpleOutput {
    //                     id: 1,
    //                     attr: Todo {
    //                         title: "todo_1".to_string(),
    //                         done: true,
    //                         description: None,
    //                     },
    //                 },
    //                 SimpleOutput {
    //                     id: 2,
    //                     attr: Todo {
    //                         title: "todo_2".to_string(),
    //                         done: false,
    //                         description: None,
    //                     },
    //                 },
    //             ],)
    //         }),))
    //     })
    // );
}
