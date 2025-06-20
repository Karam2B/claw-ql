use crate::{
    QueryBuilder,
    collections::{ OnMigrate},
};
use sqlx::Executor;
use super::LinkData;

#[derive(Clone)]
pub struct Relation<From, To> {
    pub from: From,
    pub to: To,
}

impl<S, F, T> OnMigrate<S> for Relation<F, T>
where
    Relation<F, T>: LinkData<F, Spec: OnMigrate<S>>,
    Relation<F, T>: Clone,
    F: Clone,
{
    async fn custom_migration<'e>(&self, exec: impl for<'q> Executor<'q, Database = S> + Clone)
    where
        S: QueryBuilder,
    {
        let relation = self.clone();
        let spec = relation.spec(self.from.clone());
        spec.custom_migration(exec).await;
    }
}

// todo: add meta information here!!
#[derive(Debug, Clone)]
pub struct RelationEntry {
    pub from: String,
    pub to: String,
    /// like 'many_to_many<..>', 'one_to_many<..>', etc. there is no way to have this unique for now.
    pub ty: String,
}

pub struct RelationEntries {
    pub entries: Vec<RelationEntry>,
    #[allow(unused)]
    private_to_construct_hack: (),
}

