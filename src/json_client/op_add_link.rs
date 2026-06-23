use std::sync::Arc;

use crate::{
    database_extention::DatabaseExt,
    fix_executor::ExecutorTrait,
    json_client::{
        client_interface::{AddLinkError, AddLinkInput, AddLinkOutput},
        dynamic_collection::DynamicCollection,
        sqlx_executor::{FromTo, SqlxExecutorData},
    },
    links::{
        DefaultRelationKey, relation_many_to_many::ManyToMany,
        relation_optional_to_many::OptionalToMany, timestamp::Timestamp,
    },
    on_migrate::OnMigrate,
    sqlx_query_builder::{Expression, StatementBuilder},
};

pub fn add_link<S>(
    this: Arc<SqlxExecutorData<S>>,
    input: AddLinkInput,
) -> impl Future<Output = Result<AddLinkOutput, AddLinkError>> + 'static + Send + use<S>
where
    S: DatabaseExt + Sync + Send + ExecutorTrait,
    OptionalToMany<DefaultRelationKey, Arc<DynamicCollection<S>>, Arc<DynamicCollection<S>>>:
        OnMigrate<Statements: for<'q> Expression<'q, S>>,
    ManyToMany<DefaultRelationKey, Arc<DynamicCollection<S>>, Arc<DynamicCollection<S>>>:
        OnMigrate<Statements: for<'q> Expression<'q, S>>,
    Timestamp<Arc<DynamicCollection<S>>>: OnMigrate<Statements: for<'q> Expression<'q, S>>,
{
    async move {
        match input {
            AddLinkInput::OptionalToMany { from, to } => {
                {
                    let li_read = this.link_info.read().await;
                    if li_read.optional_to_many.contains(&FromTo {
                        from: from.detach(),
                        to: to.detach(),
                    }) {
                        return Err(AddLinkError::LinkAlreadyExists);
                    }
                }

                let collections = this.collections.read().await;
                let from_col = collections
                    .get(from.as_str())
                    .ok_or(AddLinkError::CollectionNotFound)?
                    .read()
                    .await
                    .clone();
                let to_col = collections
                    .get(to.as_str())
                    .ok_or(AddLinkError::CollectionNotFound)?
                    .read()
                    .await
                    .clone();
                drop(collections);

                let mig =
                    StatementBuilder::<S>::new_no_data(OnMigrate::statments(&OptionalToMany {
                        fk_unique_id: DefaultRelationKey,
                        from: from_col,
                        to: to_col,
                    }))
                    .expect("bug: {}");

                let mut conn = this.pool.acquire().await.unwrap();
                S::execute(&mut *conn, mig.as_str()).await.unwrap();

                let mut migration = this.migration.write().await;
                let mut li_write = this.link_info.write().await;
                migration.push(mig);
                li_write.optional_to_many.insert(FromTo {
                    from: from.detach(),
                    to: to.detach(),
                });

                Ok(())
            }
            AddLinkInput::ManyToMany { from, to } => {
                {
                    let li_read = this.link_info.read().await;
                    if li_read.many_to_many.contains(&FromTo {
                        from: from.detach(),
                        to: to.detach(),
                    }) {
                        return Err(AddLinkError::LinkAlreadyExists);
                    }
                }

                let collections = this.collections.read().await;
                let from_col = collections
                    .get(from.as_str())
                    .ok_or(AddLinkError::CollectionNotFound)?
                    .read()
                    .await
                    .clone();
                let to_col = collections
                    .get(to.as_str())
                    .ok_or(AddLinkError::CollectionNotFound)?
                    .read()
                    .await
                    .clone();
                drop(collections);

                let mig = StatementBuilder::<S>::new_no_data(OnMigrate::statments(&ManyToMany {
                    relation_key: DefaultRelationKey,
                    from: from_col,
                    to: to_col,
                }))
                .expect("bug: many_to_many migration contains bind parameters");

                let mut conn = this.pool.acquire().await.unwrap();
                S::execute(&mut *conn, mig.as_str()).await.unwrap();

                let mut migration = this.migration.write().await;
                let mut li_write = this.link_info.write().await;
                migration.push(mig);
                li_write.many_to_many.insert(FromTo {
                    from: from.detach(),
                    to: to.detach(),
                });

                Ok(())
            }
            AddLinkInput::Timestamp { collection } => {
                {
                    let li_read = this.link_info.read().await;
                    if li_read.timestamped.contains(collection.as_str()) {
                        return Err(AddLinkError::LinkAlreadyExists);
                    }
                }

                let collections = this.collections.read().await;
                let col = collections
                    .get(collection.as_str())
                    .ok_or(AddLinkError::CollectionNotFound)?
                    .read()
                    .await
                    .clone();
                drop(collections);

                let mig = StatementBuilder::<S>::new_no_data(OnMigrate::statments(&Timestamp {
                    collection: col,
                }))
                .expect("bug: timestamp migration contains bind parameters");

                let mut conn = this
                    .pool
                    .acquire()
                    .await
                    .expect("dev_ops: acquire connection");
                S::execute(&mut *conn, mig.as_str())
                    .await
                    .expect("bug: timestamp migration failed");

                let mut migration = this.migration.write().await;
                let mut link_info = this.link_info.write().await;
                migration.push(mig);
                link_info.timestamped.insert(collection.detach());

                Ok(())
            }
        }
    }
}
