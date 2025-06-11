use super::relation_many_to_many::ManyToMany;
use super::{LinkData, relation::Relation, relation_optional_to_many::OptionalToMany};
use crate::collections::{Collection, CollectionBasic};
use crate::execute::Execute;
use crate::operations::CollectionOutput;
use crate::{
    QueryBuilder, collections::HasHandler, operations::insert_one_op::InsertOneFragment,
    prelude::stmt::InsertOneSt,
};
use sqlx::Row;
use sqlx::{ColumnIndex, Decode, Encode, prelude::Type};

pub struct SetNew<Input> {
    pub input: Input,
}

pub trait SetNewTrait<T1, T2> {
    type Input;
}

pub struct SetNewSpec<Relation, Input> {
    relation: Relation,
    input: Option<Input>,
}

#[rustfmt::skip]
impl<C, To: HasHandler> LinkData<C> for SetNew<To>
where
    C: Clone,
    To::Handler: Clone,
    Relation<C, To::Handler>: 
        LinkData<
            C, 
            Spec: SetNewTrait<C, To::Handler, Input = To>
        >,
{
    type Spec = SetNewSpec<<Relation<C, To::Handler> as LinkData<C>>::Spec, To>;

    fn spec(self, from: C) -> Self::Spec
    where
        Self: Sized,
    {
        let relation = Relation {
            from: from.clone(),
            to: To::Handler::default()
        }
        .spec(from);
        SetNewSpec {
            relation,
            input: Some(self.input),
        }
    }
}

impl<C, To: HasHandler> LinkData<C> for SetNew<Vec<To>>
where
    C: Clone,
    To::Handler: Clone,
    Relation<C, To::Handler>: LinkData<C, Spec: SetNewTrait<C, To::Handler, Input = Vec<To>>>,
{
    type Spec = SetNewSpec<<Relation<C, To::Handler> as LinkData<C>>::Spec, Vec<To>>;

    fn spec(self, from: C) -> Self::Spec
    where
        Self: Sized,
    {
        let relation = Relation {
            from: from.clone(),
            to: To::Handler::default(),
        }
        .spec(from);
        SetNewSpec {
            relation,
            input: Some(self.input),
        }
    }
}

impl<T1, T2: CollectionBasic> SetNewTrait<T1, T2> for OptionalToMany<T1, T2> {
    type Input = T2::LinkedData;
}

impl<S, C, To> InsertOneFragment<S> for SetNewSpec<OptionalToMany<C, To>, To::LinkedData>
where
    C: Collection<S>,
    To: Collection<S>,
    To: CollectionBasic<LinkedData: Send + Sync>,
    S: QueryBuilder,
    To: Send + Sync,
    C: Send + Sync,
    i64: Type<S> + for<'e> Encode<'e, S> + for<'e> Decode<'e, S>,
    for<'s> &'s str: ColumnIndex<S::Row>,
{
    type Inner = (Option<To::LinkedData>, Option<i64>);

    type Output = CollectionOutput<To::LinkedData>;

    fn on_insert(&mut self, data: &mut Self::Inner, st: &mut InsertOneSt<S>) {
        st.col(self.relation.foriegn_key.clone(), data.1.unwrap())
    }

    fn returning(&mut self) -> Vec<String> {
        /* no op: I already have the foriegn_key value */
        vec![]
    }

    fn from_row(&mut self, _data: &mut Self::Inner, _row: &S::Row) {
        /* no op: I already have the foriegn_key value */
    }

    fn second_sub_op<'this, E: for<'q> sqlx::Executor<'q, Database = S> + Clone>(
        &'this mut self,
        _data: &'this mut Self::Inner,
        _exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, To, C, S, E> {
        async { /* no-op */ }
    }
    fn first_sub_op<'this, E: for<'q> sqlx::Executor<'q, Database = S> + Clone>(
        &'this mut self,
        data: &'this mut Self::Inner,
        exec: E,
    ) -> impl Future<Output = ()> + Send + use<'this, To, C, S, E> {
        async {
            let mut st = InsertOneSt::init(self.relation.to.table_name().to_string());
            self.relation.to.on_insert(
                self.input
                    .take()
                    .expect("input should be initialized as Some(*) and taken only once"),
                &mut st,
            );

            let mut members = self.relation.to.members();
            members.extend(["id".to_string()]);
            st.returning(members)
                .fetch_one(exec, |r| {
                    *data = (
                        Some(self.relation.to.from_row_noscope(&r)),
                        Some(r.get("id")),
                    );
                    Ok(())
                })
                .await
                .unwrap();
        }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        CollectionOutput {
            id: data.1.unwrap(),
            attr: data.0.unwrap(),
        }
    }
}

impl<T1, T2: CollectionBasic> SetNewTrait<T1, T2> for ManyToMany<T1, T2> {
    type Input = Vec<T2::LinkedData>;
}

// impl<S, C, To> InsertOneFragment<S> for SetNewSpec<ManyToMany<C, To>, Vec<To::LinkedData>>
// where
//     C: Collection<S>,
//     To: Collection<S>,
//     To::LinkedData: Clone,
//     To: CollectionBasic<LinkedData: Send + Sync>,
//     S: QueryBuilder,
//     To: Send + Sync,
//     C: Send + Sync,
//     i64: Type<S> + for<'e> Encode<'e, S> + for<'e> Decode<'e, S>,
//     for<'s> &'s str: ColumnIndex<S::Row>,
// {
// }
