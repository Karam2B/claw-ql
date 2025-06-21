mod tracing_testing {
    use std::marker::PhantomData;

    use claw_ql::IntoInferFromPhantom;
    use sqlx::SqlitePool;
    use sqlx::query;
    use tracing::instrument::WithSubscriber;

    use tracing::callsite::DefaultCallsite;
    use tracing::callsite::Identifier;
    use tracing::metadata;
    use tracing::{Level, Subscriber, field::FieldSet, span};

    #[derive(Debug, PartialEq)]
    pub struct DebugMetaData(metadata::Metadata<'static>);

    pub struct DebugMetaNew {
        pub name: &'static str,
        pub target: &'static str,
        pub level: Level,
        pub file: Option<&'static str>,
        pub line: Option<u32>,
        pub module_path: Option<&'static str>,
        pub fields: &'static [&'static str],
        pub kind: metadata::Kind,
    }

    impl From<DebugMetaNew> for DebugMetaData {
        fn from(value: DebugMetaNew) -> Self {
            static __CALLSITE: DefaultCallsite = tracing::callsite2! {
                name: "callsite name",
                kind: tracing::metadata::Kind::HINT,
                fields: multi tokens
            };

            DebugMetaData(tracing::Metadata::new(
                value.name,
                value.target,
                value.level,
                value.file,
                value.line,
                value.module_path,
                FieldSet::new(
                    value.fields,
                    // debugs to Identifier(Pointer { addr, metadata: impl PartialEq })
                    Identifier(&__CALLSITE),
                ),
                value.kind,
            ))
        }
    }

    impl Subscriber for DebugMetaData {
        fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
            pretty_assertions::assert_eq!(metadata, &self.0);
            todo!("enabled is not implemented")
        }
        fn new_span(&self, _: &span::Attributes<'_>) -> span::Id {
            todo!()
        }
        fn record(&self, _: &span::Id, _: &span::Record<'_>) {
            todo!()
        }
        fn record_follows_from(&self, _: &span::Id, _: &span::Id) {
            todo!()
        }
        fn event(&self, _: &tracing::Event<'_>) {
            todo!()
        }
        fn enter(&self, _: &span::Id) {
            todo!()
        }
        fn exit(&self, _: &span::Id) {
            todo!()
        }
    }

    #[tokio::test]
    async fn main() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        let _db = DebugMetaNew {
            name: "dsf",
            target: "dsf",
            level: Level::DEBUG,
            file: None,
            line: None,
            module_path: Some("sdf"),
            fields: &["dfs"],
            kind: tracing::metadata::Kind::HINT,
        }
        .into_pd(PhantomData::<DebugMetaData>);

        async {
            query("CREATE TABLE todo(id INIT);")
                .execute(&pool)
                .await
                .unwrap();
        }
        .with_subscriber(
            tracing_subscriber::fmt()
                .with_max_level(Level::DEBUG)
                .finish(),
        )
        .await;
    }
}
