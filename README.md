# ClawQl

Robust and flexable Rust ORM.

```Rust
#[derive(Collection, OnMigrate, FromRowAlias)]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[derive(Collection, OnMigrate, FromRowAlias)]
pub struct Category {
    pub title: String,
}

impl Link<todo> for category {
    type Spec = optional_to_many<String, todo, category>;
    fn spec(self, _: &todo) -> Self::Spec {
        optional_to_many {
            foriegn_key: String::from("category_id"),
            from: todo,
            to: self,
        }
    }
}

#[tokio::test]
async fn main() {
    let pool = Sqlite::connect_in_memory().await;

    use claw_ql::expressions::col_eq;
    use claw_ql::links::relation_optional_to_many::optional_to_many;
    use claw_ql::links::set_new_mod::set_new;
    use claw_ql::test_module::{Category, Todo, category, todo, todo_members};

    sql!(MIGRATE todo).await;
    sql!(MIGRATE category).await;
    sql!(MIGRATE optional_to_many {
        from: todo,
        to: category,
        foriegn_key: "category_id".to_string()
    })
    .await;

    sql!(
        INSERT Todo { title:"first_todo".to_string(), done: false, description: None }
            LINK set_new(Category { title: "cat_1".to_string() })
    )
    .await;

    let result = sql!(
        SELECT FROM todo t
        LINK category
        WHERE t.title.col_eq("first_todo".to_string())
    )
    .await;

    pretty_assertions::assert_eq!(
        result,
        Some(LinkedOutput {
            id: 0,
            attributes: Todo {
                title: "first_todo".to_string(),
                done: false,
                description: None
            },
            links: (Some(CollectionOutput {
                id: 0,
                attributes: Category {
                    title: "cat_1".to_string()
                }
            }),),
        })
    );
}
```

`sql` macro doesn't do too much magic -- you can just construct types that implement `Operation` and call `exec_operation` on them. I made the macro to create an similar experience to SQL syntax

Note that this API is heavy on the type system, if you want to create an HTTP server, use `JsonClient`. This API rely on extension traits and trait objects to create a more dynamic/runtime experience at zero effort. 

```Rust
#[tokio::test]
async fn json_client_test() {
    let mut jc = JsonClient::from(
        (
            Schema {
                collections: (todo, category)
                links: (optional_to_many {
                    foriegn_key: "category_id".to_string(),
                    from: todo,
                    to: category,
                },)
            },
            pool,
        )
    );

    let out = jc
        .fetch_one(json!({
            "base": "todo",
            "wheres": [],
            "link": [
                {
                    "id": "category_id",
                    "ty": "optional_to_many",
                    "to": "category",
                },
            ]
        }))
        .await
        .unwrap();

    assert_eq!(
        out,
        json!({
            "attributes": {
                "title": "first_todo",
                "done": true,
                "description": "description_1"
            },
            "id": 6,
            "link": [{
                "attributes": { "title": "cat_1" },
                "id": 3,
            }]
        }),
    );
}
```

# What are links
This is the bread and butter of this crate, they use foreign keys, joins, and sessions when necessary to optimize performance, I'm not aiming to replace foreign keys -- I think storing data in tables with FKs between them is solid idea, however retrieving data as tables via string-based query is tedious and error-prone. 

I always had a dilemma whether to use the SQL client directly with hardcoded statements or use an ORM, and I figured out the problem finaly -- replace joins with links, I have a blog post talking about that in details. If there is a database that provide link-base interface, use FKs and joins internally, and you can query via something similar to BSON, this would make 90% or this crate (and most aother ORMs) unnecesary.

# I'm looking for help

this is proof of concept, I have full CRUD API written but in an older version of this crate, reimplemnting everything is straight forward, I mainly looking for time or contribution to complete.
