#![allow(unused)]
use super::{
    LinkData,
    relation::Relation,
    relation_many_to_many::ManyToMany,
    relation_optional_to_many::{OptionalToMany, OptionalToManyInverse},
};
use crate::{
    QueryBuilder, collections::CollectionBasic, operations::insert_one_op::InsertOneFragment,
    prelude::stmt::InsertOneSt,
};
use sqlx::{ColumnIndex, Decode, Encode, prelude::Type};
use sqlx::{IntoArguments, Row};
use std::{future::Future, usize};

pub struct SetId<To, Input> {
    pub input: Input,
    pub to: To,
}

pub trait SetIdTrait<T1, T2> {
    type Input;
}

pub struct SetIdSpec<Relation, Input> {
    relation: Relation,
    input: Input,
}

impl<C, To> LinkData<C> for SetId<To, i64>
where
    C: CollectionBasic,
    To: CollectionBasic,
    Relation<C, To>: LinkData<C, Spec: SetIdTrait<C, To, Input = i64>>,
{
    type Spec = SetIdSpec<<Relation<C, To> as LinkData<C>>::Spec, i64>;

    fn spec(self, from: C) -> Self::Spec
    where
        Self: Sized,
    {
        let relation = Relation {
            from: from.clone(),
            to: self.to.clone(),
        }
        .spec(from);
        SetIdSpec {
            relation,
            input: self.input,
        }
    }
}
impl<C, To> LinkData<C> for SetId<To, Vec<i64>>
where
    C: CollectionBasic,
    To: CollectionBasic,
    Relation<C, To>: LinkData<C, Spec: SetIdTrait<C, To, Input = Vec<i64>>>,
{
    type Spec = SetIdSpec<<Relation<C, To> as LinkData<C>>::Spec, Vec<i64>>;

    fn spec(self, from: C) -> Self::Spec
    where
        Self: Sized,
    {
        let relation = Relation {
            from: from.clone(),
            to: self.to.clone(),
        }
        .spec(from);
        SetIdSpec {
            relation,
            input: self.input,
        }
    }
}

// optional_to_many: set local key which have foriegn key constraint
// TODO: optional_to_many_inverse: set *multiple* foriegn keys which have foriegn key constraint
impl<T1, T2> SetIdTrait<T1, T2> for OptionalToMany<T1, T2> {
    type Input = i64;
}

impl<S, C, To> InsertOneFragment<S> for SetIdSpec<OptionalToMany<C, To>, i64>
where
    S: QueryBuilder,
    To: Send + Sync,
    C: Send + Sync,
    i64: Type<S> + for<'e> Encode<'e, S> + for<'e> Decode<'e, S>,
    for<'s> &'s str: ColumnIndex<S::Row>,
{
    type Inner = Option<i64>;

    type Output = i64;

    fn on_insert(&mut self, data: &mut Self::Inner, st: &mut InsertOneSt<S>) {
        st.col(self.relation.foriegn_key.clone(), self.input);
    }

    fn returning(&mut self) -> Vec<String> {
        vec![self.relation.foriegn_key.clone()]
    }

    fn from_row(&mut self, data: &mut Self::Inner, row: &S::Row) {
        *data = Some(row.get(&*self.relation.foriegn_key))
    }

    fn first_sub_op<'this, E: for<'q> sqlx::Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, To, C, S, E> {
        async { /* no-op */ }
    }
    fn second_sub_op<'this, E: for<'q> sqlx::Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, To, C, S, E> {
        async { /* no-op */ }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        data.unwrap()
    }
}

// many_to_many: set entries in a junction table: you need from_row:id and input
impl<T1, T2> SetIdTrait<T1, T2> for ManyToMany<T1, T2> {
    type Input = Vec<i64>;
}

impl<S, C, To> InsertOneFragment<S> for SetIdSpec<ManyToMany<C, To>, Vec<i64>>
where
    for<'q> S::Arguments<'q>: IntoArguments<'q, S>,
    S: QueryBuilder,
    To: Send + Sync,
    C: Send + Sync,
    i64: Type<S> + for<'e> Encode<'e, S> + for<'e> Decode<'e, S>,
    for<'s> &'s str: ColumnIndex<S::Row>,
    usize: ColumnIndex<S::Row>,
{
    type Inner = (Option<i64>, Vec<i64>);

    type Output = Vec<i64>;

    fn on_insert(&mut self, data: &mut Self::Inner, st: &mut InsertOneSt<S>) {
        /* no op */
    }

    fn returning(&mut self) -> Vec<String> {
        /* no op assuming "id" already added! */
        vec![]
    }

    fn from_row(&mut self, data: &mut Self::Inner, row: &S::Row) {
        *&mut data.0 = Some(row.get("id"))
    }

    fn first_sub_op<'this, E: for<'q> sqlx::Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, To, C, S, E> {
        async { /* no op */ }
    }
    fn second_sub_op<'this, E: for<'q> sqlx::Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, To, C, S, E> {
        async {
            let id1 = data.0.unwrap();
            let rows: Vec<(i64,)> = sqlx::query_as(&format!(
                "
            INSERT INTO {junction} ({id1}, {id2}) 
                VALUES {}
            RETURNING {id2};
               
    ",
                self.input
                    .iter()
                    .map(|id2| { format!("({id1}, {id2})") })
                    .collect::<Vec<_>>()
                    .join(", "),
                junction = self.relation.junction,
                id1 = self.relation.id_1,
                id2 = self.relation.id_2,
            ))
            .fetch_all(exec)
            .await
            .unwrap();

            *&mut data.1 = rows.into_iter().map(|e| e.0).collect();
        }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        data.1
    }
}
