use std::{iter::StepBy, marker::PhantomData};

use sqlx::{Arguments, Database, Encode, Type, sqlite::SqliteArguments};

use crate::{EncodeExtention, Expression, ExpressionToFragment, QueryBuilder, SanitzingMechanisim};

pub struct direct_bind<'q, S: Database> {
    pub increment: usize,
    pub arg: S::Arguments<'q>,
    pub db: PhantomData<S>,
}

impl<'q, S: Database> direct_bind<'q, S> {
    pub fn new(db: S) -> Self {
        Self {
            increment: Default::default(),
            arg: Default::default(),
            db: PhantomData,
        }
    }
}

impl<'q, S: Database> Default for direct_bind<'q, S> {
    fn default() -> Self {
        Self {
            increment: Default::default(),
            arg: Default::default(),
            db: PhantomData,
        }
    }
}

impl<'q, S: Database> QueryBuilder for direct_bind<'q, S> {
    type Fragment = String;
    type Context = ();
    type SqlxDb = S;
    type Output = S::Arguments<'q>;

    fn fragment_to_string(ctx: &mut Self::Context, from: Self::Fragment) -> String {
        from
    }
    fn to_output(
        self,
        statement_builder: impl FnOnce(&mut Self::Context) -> String,
    ) -> (String, Self::Output) {
        (statement_builder(&mut ()), self.arg)
    }
}

impl<'q, S: Database, T> EncodeExtention<'q, T> for direct_bind<'q, S>
where
    T: Encode<'q, S> + Type<S> + Send + 'q,
{
    fn encode(&mut self, val: T) -> impl FnOnce(&mut Self::Context) -> String + 'q + use<'q, T, S> {
        self.arg.add(val).expect("bug maybe?");
        self.increment += 1;
        let increment = self.increment;
        move |_| format!("${}", increment)
    }
}

impl<'q, E, S> ExpressionToFragment<'q, E> for direct_bind<'q, S>
where
    E: Expression<'q, Self>,
    S: Database,
{
    fn expression_to_fragment(&mut self, t: E) -> <Self as QueryBuilder>::Fragment {
        t.expression(self)(&mut ())
    }
}

impl<'q, S> SanitzingMechanisim for direct_bind<'q, S>
where
    S: Database + SanitzingMechanisim,
{
    type SanitzingMechanisim = S::SanitzingMechanisim;
}
