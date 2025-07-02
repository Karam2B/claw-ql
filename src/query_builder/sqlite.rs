use sqlx::{Database, Encode, Sqlite, Type};

use crate::{Accept, QueryBuilder};

pub struct QuickQueryCtx<S: Database> {
    size: usize,
    arg: S::Arguments<'static>,
}

impl<S: Database> Default for QuickQueryCtx<S> {
    fn default() -> Self {
        QuickQueryCtx {
            size: 0,
            arg: Default::default(),
        }
    }
}

impl<S: Database> From<QuickQueryCtx<S>> for () {
    fn from(_this: QuickQueryCtx<S>) -> Self {
        ()
    }
}

impl QueryBuilder for sqlx::Sqlite {
    type Fragment = String;

    type Context1 = QuickQueryCtx<Self>;

    type Context2 = ();

    fn build_sql_part_back(_ctx: &mut Self::Context2, from: Self::Fragment) -> String {
        from
    }

    type Output = Self::Arguments<'static>;

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        // let noop = unsafe { &mut *(&mut ctx1.noop as *mut _) };
        let strr = f(&mut ());
        (strr, ctx1.arg)
    }

    fn handle_bind_item<T>(t: T, ctx: &mut Self::Context1) -> Self::Fragment
    where
        T: crate::BindItem<Self> + 'static,
    {
        t.bind_item(ctx)(&mut ())
    }

    fn handle_accept<T>(t: T, ctx: &mut Self::Context1) -> Self::Fragment
    where
        T: 'static + Send,
        Self: crate::Accept<T>,
    {
        Self::accept(t, ctx)(&mut ())
    }
}
impl<T> Accept<T> for Sqlite
where
    T: for<'e> Encode<'e, Sqlite> + Type<Sqlite> + Send + 'static,
{
    fn accept(
        this: T,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send + use<T> {
        use sqlx::Arguments;
        ctx1.arg.add(this).unwrap();
        ctx1.size += 1;
        let len = ctx1.size;
        move |_| format!("${}", len)
    }
}
