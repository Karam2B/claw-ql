#![allow(unused)]
#![deny(unused_must_use)]

use claw_ql::{
    ConnectInMemory, Schema,
    migration::{MigrationStep, OnMigrate, create_from_scrach_migration_step, migrate},
};
use claw_ql_macros::Collection;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Pool, Sqlite};

#[tokio::test]
async fn foo() {
    let pool = Sqlite::connect_in_memory().await;

    mod v1 {
        use claw_ql_macros::Collection;
        use serde::{Deserialize, Serialize};

        #[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
        pub struct Todo {
            pub title: String,
        }
    }
    mod v2 {
        use claw_ql_macros::Collection;
        use serde::{Deserialize, Serialize};

        #[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
        pub struct Todo {
            // did I renmae title or created a new field and deleted the old field?
            pub title2: String,
        }
    }

    let schema = Schema {
        collections: (v1::todo,),
        links: (),
    };

    migrate(
        vec![MigrationStep {
            version: 0,
            sql: vec![],
        }],
        &pool,
    )
    .await;

    #[derive(Collection, Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct Todo {
        // did I renmae title or created a new field and deleted the old field?
        pub title2: String,
    }

    let schema = Schema {
        collections: (v2::todo,),
        links: (),
    };

    migrate(
        vec![
            MigrationStep {
                version: 1,
                sql: vec!["ALTER TABLE todo RENAME COLUMN title TO title2".to_string()],
            },
            create_from_scrach_migration_step::<Sqlite, _, _>(schema.clone()),
        ],
        &pool,
    )
    .await;

    // alternative is that I deleted the old field and created a new field
    migrate(
        vec![
            MigrationStep {
                version: 1,
                sql: vec![
                    "ALTER TABLE todo DROP COLUMN title".to_string(),
                    "ALTER TABLE todo ADD COLUMN title2 TEXT".to_string(),
                ],
            },
            create_from_scrach_migration_step::<Sqlite, _, _>(schema.clone()),
        ],
        &pool,
    )
    .await;
}
