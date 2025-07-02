use crate::{
    json_client::{DynamicLinkBTDyn, JsonCollection},
    migration::OnMigrateDyn,
};
use inventory::collect;
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
