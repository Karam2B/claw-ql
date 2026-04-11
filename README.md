this is Rust ORM

```
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

    { ...  /* migration and dumpy data */ }

    let fetch_one = sql!(
        SELECT FROM todo
        LINK category
        WHERE title.eq("first_todo")
    )
    .await;

    assert_eq!(
        fetch_one,
        LinkedOutput {
            id: 6,
            attributes: Todo {
                title: "first_todo".to_string(),
                done: true,
                description: Some("description_1".to_string())
            },
            link: (CollectionOutput {
                id: 3,
                attributes: Category {
                    title: "cat_1".to_string()
                }
            },)
        }
    );
}
```

or you can create a dynamic json client that is suitable for http servers

```
#[tokio::test]
async fn json_client_test() {
    let mut jc = JsonClient {
        ..from_schema(
            Schema {
                collections: (todo, category)
                links: (optional_to_many {
                    foriegn_key: "category_id".to_string(),
                    from: todo,
                    to: category,
                },)
            }
        )
        pool,
    };

    let out = jc
        .fetch_one_json(json!({
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

you can migrate 

```
    migrate_on_empty_database(
        vec![
            Box::new(todo),
            Box::new(category),
            Box::new(optional_to_many {
                foriegn_key: "category_id".to_string(),
                from: todo,
                to: category,
            }),
        ],
        &pool,
    )
    .await;

    dumy_data(pool).await;
```

this is prove of concept, I have full crud-api written but in an older version of this crate, writing the rest of operation is straight forward, I just want time or contribution to be implement

why not joins? I think the main idea of sql in genius (data stored in talbles with foriegn keys) but I think sql is not good at fetch data from db, because it force data in to tables, which is not how we usually build UI for example

I alawy had a concodrum which is when should I use sql directly and when I should use ORM