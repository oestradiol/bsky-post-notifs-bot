mod get_watched_users;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use types::entities::watched_user::Watcher;

use std::{
  collections::{BTreeMap, HashSet},
  sync::Arc,
};

use async_once::AsyncOnce;
use lazy_static::lazy_static;

lazy_static! {
  pub static ref WATCHED_USERS: AsyncOnce<RwLock<BTreeMap<Arc<str>, Data>>> =
    AsyncOnce::new(init_watched_users());
}

#[allow(clippy::unused_async)] // TODO: Remove this once the function is actually used
async fn init_watched_users() -> RwLock<BTreeMap<Arc<str>, Data>> {
  RwLock::new(get_watched_users::act().await)
}

pub struct Data {
  pub last_notified_watchers: DateTime<Utc>,
  pub watchers: HashSet<Watcher>,
}
