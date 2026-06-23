use std::sync::Arc;

use sqlx::ColumnIndex;

use crate::{
    collections::Collection,
    database_extention::DatabaseExt,
    expressions::ColumnEqual,
    extentions::common_expressions::Scoped,
    fix_executor::ExecutorTrait,
    from_row::FromRowAlias,
    json_client::{
        DynManyToMany, DynOptionalToMany, DynOptionalToManyInverse, DynTimestamp,
        client_interface::{FetchOneError, FetchOneInput, FetchOneOutput, SupportedLinkFetchOne},
        dynamic_collection::{CollectionToSerialize, DynamicCollection},
        op_fetch_one_trait_extension::JsonLinkFetchOne,
        sqlx_executor::{FromTo, SqlxExecutorData},
        supported_filters::parse_supported_filter,
    },
    links::{
        DefaultRelationKey, relation_many_to_many::ManyToMany,
        relation_optional_to_many::OptionalToMany,
        relation_optional_to_many_inverse::OptionalToManyInverse, timestamp::Timestamp,
    },
    operations::{Operation, fetch_one::FetchOne},
    sqlx_query_builder::basic_expressions::ManyFlat,
};

type DynCollection<S> = Arc<DynamicCollection<S>>;

pub fn fetch_one<S>(
    this: Arc<SqlxExecutorData<S>>,
    input: FetchOneInput,
) -> impl Future<Output = Result<FetchOneOutput, FetchOneError>> + 'static + Send + use<S>
where
    S: DatabaseExt + ExecutorTrait + Send + Sync,
    DynCollection<S>: for<'r> FromRowAlias<'r, S::Row, RData = CollectionToSerialize>,
    DynOptionalToMany<S>: JsonLinkFetchOne<S>,
    DynOptionalToManyInverse<S>: JsonLinkFetchOne<S>,
    DynManyToMany<S>: JsonLinkFetchOne<S>,
    DynTimestamp<S>: JsonLinkFetchOne<S>,
    i64: sqlx::Type<S> + for<'q> sqlx::Decode<'q, S> + for<'q> sqlx::Encode<'q, S>,
    String: for<'q> sqlx::Encode<'q, S> + sqlx::Type<S>,
    for<'a> &'a str: ColumnIndex<S::Row>,
{
    async move {
        let cols = this.collections.read().await;
        let base_guard = cols
            .get(input.base.as_str())
            .ok_or(FetchOneError::CollectionNotFound)?
            .read()
            .await;
        let base = base_guard.clone();
        let rel_guard = this.link_info.read().await;
        let mut all_guards = vec![base_guard];

        let filter_exprs = parse_supported_filter(input.filters, &base)
            .map_err(|_| FetchOneError::InvalidFilter)?;

        let mut links = Vec::<Box<dyn JsonLinkFetchOne<S> + Send>>::new();

        for each in input.links {
            match each {
                SupportedLinkFetchOne::OptionalToMany { to } => {
                    let to_guard = cols
                        .get(to.as_str())
                        .ok_or(FetchOneError::InvalidLink)?
                        .read()
                        .await;
                    let to = to_guard.clone();
                    all_guards.push(to_guard);

                    let forward = FromTo {
                        from: Arc::clone(&base.collection_name.snake_case),
                        to: Arc::clone(&to.collection_name.snake_case),
                    };
                    let reverse = FromTo {
                        from: Arc::clone(&to.collection_name.snake_case),
                        to: Arc::clone(&base.collection_name.snake_case),
                    };

                    if rel_guard.optional_to_many.contains(&forward) {
                        links.push(Box::new(OptionalToMany {
                            fk_unique_id: DefaultRelationKey,
                            from: Arc::clone(&base),
                            to,
                        }));
                    } else if rel_guard.optional_to_many.contains(&reverse) {
                        links.push(Box::new(OptionalToManyInverse {
                            fk_unique_id: DefaultRelationKey,
                            from: Arc::clone(&base),
                            to,
                        }));
                    } else {
                        return Err(FetchOneError::InvalidLink);
                    }
                }
                SupportedLinkFetchOne::ManyToMany { to } => {
                    let to_guard = cols
                        .get(to.as_str())
                        .ok_or(FetchOneError::InvalidLink)?
                        .read()
                        .await;
                    let to = to_guard.clone();
                    all_guards.push(to_guard);

                    let forward = FromTo {
                        from: Arc::clone(&base.collection_name.snake_case),
                        to: Arc::clone(&to.collection_name.snake_case),
                    };

                    if rel_guard.many_to_many.contains(&forward) {
                        links.push(Box::new(ManyToMany {
                            relation_key: DefaultRelationKey,
                            from: Arc::clone(&base),
                            to,
                        }));
                    } else {
                        return Err(FetchOneError::InvalidLink);
                    }
                }
                SupportedLinkFetchOne::Timestamp => {
                    if !rel_guard
                        .timestamped
                        .contains(base.collection_name.snake_case.as_ref())
                    {
                        return Err(FetchOneError::InvalidLink);
                    }
                    links.push(Box::new(Timestamp {
                        collection: Arc::clone(&base),
                    }));
                }
            }
        }

        let wheres = ManyFlat((
            ColumnEqual {
                col: base.id().scoped(),
                eq: input.id,
            },
            ManyFlat(filter_exprs),
        ));

        let mut conn = this.pool.acquire().await.unwrap();

        let out = Operation::<S>::exec_operation(
            FetchOne {
                base,
                wheres,
                links,
            },
            &mut conn,
        )
        .await;

        drop(rel_guard);
        drop(all_guards);
        drop(cols);

        out.ok_or(FetchOneError::NotFound)
    }
}
