ORM built in rust, focused on performance and flexibility.

```rust
#[derive(Collection)]
struct Todo {
    title: String,
    done: bool,
    description: Option<String>,
}

#[derive(Collection)]
struct Category {
    title: String,
}

impl Link<todo> for category {
    type Spec = OptionalToMany<DefaultRealtionKey, todo, category>;
    fn spec(self) -> Self::Spec {
        OptionalToMany {
            key: DefaultRealtionKey,
            from: todo,
            to: category,
        }
    }
}

#[tokio::test]
async fn example() {
    let mut conn = SqliteConnection::connect("sqlite::memory:").await;

    InsertOne{
        handler: todo,
        data: Todo {
            title: "Buy groceries".to_string(),
            done: false,
            description: None,
        },
        links: SetNew {
            input: Category {
                title: "Home".to_string(),
            },
        },
    }.exec_op(&conn).await;

    FetchOne {
        handler: todo,
        filters: ById(1),
        links: category
    }.exec_op(&conn).await;

    assert_eq!(res, CollectionOutput {
        id: 1,
        attr: Todo {
            title: "Buy groceries".to_string(),
            done: false,
            description: None,
        },
        links: Category {
            title: "Home".to_string(),
        },
    });
}
```
