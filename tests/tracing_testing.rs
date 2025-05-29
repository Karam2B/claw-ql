use sqlx::SqlitePool;

use tracing::callsite::DefaultCallsite;
use tracing::{Level, Subscriber, field::FieldSet, span};

#[derive(Debug, PartialEq)]
pub struct MySub;

impl Subscriber for MySub {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        //     use tracing::__macro_support::Callsite as _;
        //     static __CALLSITE:tracing::callsite::DefaultCallsite = tracing::callsite2!{
        //         name:tracing::__macro_support::concat!("event ",tracing::__macro_support::file!(),":",tracing::__macro_support::line!()),kind:tracing::metadata::Kind::EVENT,target:(module_path!()),level:(tracing::Level::DEBUG),fields:"hello world"
        //     };
        //     let enabled = tracing::level_enabled!((tracing::Level::DEBUG))&&{
        //         let interest = __CALLSITE.interest();
        //         !interest.is_never()&&tracing::__macro_support::__is_enabled(__CALLSITE.metadata(),interest)
        //     };
        //     if enabled {
        //         (|value_set:tracing::field::ValueSet|{
        //             let meta = __CALLSITE.metadata();
        //             tracing::Event::dispatch(meta, &value_set);
        //             tracing::__tracing_log!((tracing::Level::DEBUG),__CALLSITE, &value_set);
        //         })(tracing::valueset!(__CALLSITE.metadata().fields(),"hello world"));
        //     }else {
        //         tracing::__tracing_log!((tracing::Level::DEBUG),__CALLSITE, &tracing::valueset!(__CALLSITE.metadata().fields(),"hello world"));
        //     }
        // };
        // let hi: DefaultCallsite = tracing::callsite2!(
        //     name:"event file:69"
        // );
        use tracing::callsite::Identifier;

        let ident: DefaultCallsite = todo!();
        let md = tracing::Metadata::new(
            /* name */ "name",
            /* target */ "target",
            /* level */ Level::DEBUG,
            /* module_path */ Some("module_path"),
            /* file */ Some(69),
            /* line */ Some("line"),
            /* fields */ FieldSet::new(&["field1, field2"], Identifier(&ident)),
            /* kind */ tracing::metadata::Kind::HINT,
        );

        pretty_assertions::assert_eq!(&md, metadata);

        todo!("{metadata:?} enabled is not implemented")
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        todo!("new_span is not implemented")
    }

    fn record(&self, span: &span::Id, values: &span::Record<'_>) {
        todo!("record is not implemented")
    }

    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {
        todo!("record_follows_from is not implemented")
    }

    fn event(&self, event: &tracing::Event<'_>) {
        todo!("event is not implemented")
    }

    fn enter(&self, span: &span::Id) {
        todo!("enter is not implemented")
    }

    fn exit(&self, span: &span::Id) {
        todo!("exit is not implemented")
    }
}

#[cfg(test)]
#[tokio::test]
async fn main() {
    use sqlx::query;
    use tracing_subscriber::util::SubscriberInitExt;

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    MySub.init();

    query("hello world").execute(&pool).await.unwrap();
}
