use crate::{
    collections::HasHandler,
    links::{Link, LinkedViaId},
};

#[allow(non_camel_case_types)]
pub struct set_new<E>(pub E);

impl<From, Entry> Link<From> for set_new<Entry>
where
    // linkedviaids should have different set_new!
    Entry::Handler: Link<From, Spec: LinkedViaId>,
    Entry: HasHandler,
    From: Clone,
    Entry::Handler: Default,
{
    type Spec = SetNewSpec<<Entry::Handler as Link<From>>::Spec, From, Entry::Handler, Entry>;
    fn spec(self, base: &From) -> Self::Spec {
        let to = Entry::Handler::default();
        let og_spec = to.spec(base);
        SetNewSpec {
            og_spec,
            from_handler: base.clone(),
            to_handler: Default::default(),
            entry: self.0,
        }
    }
}

pub struct SetNewSpec<OgSpec, From, To, Entry> {
    pub og_spec: OgSpec,
    pub from_handler: From,
    pub to_handler: To,
    pub entry: Entry,
}

mod impl_on_update {
    use super::SetNewSpec;
    use crate::{
        operations::{insert_one::LinkInsertOne, update_one::LinkUpdateOne},
        prelude::sql::InsertOne,
    };

    impl<S> LinkInsertOne<S> for () {
        type PreOp = ();

        type Resume = ();

        fn pre_op(self) -> (Self::PreOp, Self::Resume) {
            todo!()
        }

        fn current_table(_: <Self::PreOp as crate::operations::Operation<S>>::Output) -> Vec<String>
        where
            Self::PreOp: crate::operations::Operation<S>,
        {
            todo!()
        }

        type FromRow = ();

        type SubOp = ();

        fn from_row(&self, _: &S::Row) -> (Self::FromRow, Self::SubOp)
        where
            S: sqlx::Database,
        {
            todo!()
        }

        type Output = ();

        fn take(
            self,
            _: Self::FromRow,
            _: <Self::SubOp as crate::operations::Operation<S>>::Output,
        ) -> Self::Output
        where
            Self::SubOp: crate::operations::Operation<S>,
            S: sqlx::Database,
        {
            todo!()
        }
    }

    impl<S, OgSpec, From, To, Entry> LinkUpdateOne<S> for SetNewSpec<OgSpec, From, To, Entry> {
        type PreOp = InsertOne<To, Entry, ()>;
        fn pre_op(self) -> Self::PreOp {
            InsertOne {
                handler: self.to_handler,
                entry: self.entry,
                links: (),
            }
        }
    }
}

mod impl_on_insert {
    use super::SetNewSpec;
    use crate::{
        operations::{Operation, insert_one::LinkInsertOne},
        prelude::sql::InsertOne,
    };
    impl<S, OgSpec, From, To, Entry> LinkInsertOne<S> for SetNewSpec<OgSpec, From, To, Entry>
    // where
    //     InsertOne<From, Entry, ()>: Operation<S, Output = LinkedOutput<i64, (), ()>>,
    {
        type PreOp = InsertOne<From, Entry, ()>;
        type Resume = ();
        fn pre_op(self) -> (Self::PreOp, Self::Resume) {
            (
                InsertOne {
                    handler: self.from_handler,
                    entry: self.entry,
                    links: (),
                },
                (),
            )
        }

        fn current_table(o: <Self::PreOp as Operation<S>>::Output) -> Vec<String>
        where
            Self::PreOp: Operation<S>,
        {
            // let id: LinkedOutput<i64, (), ()> = o;
            todo!()
        }

        type FromRow = ();

        type SubOp = ();

        fn from_row(&self, row: &S::Row) -> (Self::FromRow, Self::SubOp)
        where
            S: sqlx::Database,
        {
            todo!()
        }

        type Output = ();

        fn take(
            self,
            from_row: Self::FromRow,
            sub_op: <Self::SubOp as crate::operations::Operation<S>>::Output,
        ) -> Self::Output
        where
            Self::SubOp: crate::operations::Operation<S>,
            S: sqlx::Database,
        {
            todo!()
        }
    }
}
