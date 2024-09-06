mod get_all;
use get_all::get_all;

mod delete;
pub use delete::*;

mod update;
pub use update::*;

mod remove_if_found;
pub use remove_if_found::*;

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

async fn init_watched_users() -> RwLock<BTreeMap<Arc<str>, Data>> {
  RwLock::new(get_all().await.unwrap_or_else(|e| panic!("Failed to get initial state for watched users! Error: {e}")))
}

pub struct Data {
  pub last_notified_watchers: DateTime<Utc>,
  pub watchers: HashSet<Watcher>,
}