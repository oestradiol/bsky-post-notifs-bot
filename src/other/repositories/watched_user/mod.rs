mod get_watched_users;
use sqlx::types::chrono::NaiveDateTime;
use tokio::sync::RwLock;
use types::entities::watched_user::Watcher;

use std::{collections::BTreeMap, sync::Arc};

use async_once::AsyncOnce;
use lazy_static::lazy_static;

lazy_static! {
  pub static ref WATCHED_USERS: AsyncOnce<RwLock<BTreeMap<Arc<str>, Data>>> =
    AsyncOnce::new(async { init_watched_users() });
}

#[must_use]
pub fn init_watched_users() -> RwLock<BTreeMap<Arc<str>, Data>> {
  RwLock::new(get_watched_users::act())
}

pub struct Data {
  pub last_peeked: NaiveDateTime,
  pub watchers: Vec<Watcher>,
}
