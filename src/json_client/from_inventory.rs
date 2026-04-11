use crate::json_client::JsonCollection;
use sqlx::Any as SqlxAny;
use std::{any::Any, sync::Arc};

pub struct Collection {
    pub obj: fn() -> Arc<dyn JsonCollection<SqlxAny>>,
}

collect!(Collection);

pub struct Migration {
    pub obj: fn() -> Box<dyn OnMigrateDyn<SqlxAny>>,
}

collect!(Migration);

pub struct Link {
    pub obj: fn() -> Box<dyn DynamicLinkBTDyn<SqlxAny>>,
}

collect!(Link);

#[cfg(feature = "inventory")]
impl JsonClient<sqlx::Any> {
    pub fn new_from_inventory(
        db: Pool<sqlx::Any>,
    ) -> BuilderPattern<PhantomData<(to_json_client<sqlx::Any>,)>, (JsonClientBuilding<sqlx::Any>,)>
    {
        use crate::inventory::{Collection, Link};
        use convert_case::{Case, Casing};

        let mut b = JsonClientBuilding {
            collections: Default::default(),
            links: Default::default(),
            flex_ctx: Default::default(),
            db,
        };

        for coll in inventory::iter::<Collection> {
            let coll = (coll.obj)();
            let name = coll.table_name();
            let ret = b.collections.insert(name.to_case(Case::Snake), coll);

            if ret.is_some() {
                panic!(
                    "collections are globally unique, the identifier {} was used twice",
                    name
                )
            }
        }

        for link in inventory::iter::<Link> {
            let obj = (link.obj)();
            let meta = obj.buildtime_meta();

            b.flex_ctx.push(meta);

            let mut more = obj.push_more();
            while more.is_some() {
                let more_inner = more.unwrap();

                let buildtime_meta = more_inner.buildtime_meta();
                b.flex_ctx.push(Box::new(buildtime_meta));

                more = more_inner.push_more();

                b.links.push(more_inner);
            }

            b.links.push(obj);
        }

        BuilderPattern {
            __components: PhantomData,
            __context: (b,),
        }
    }
}
