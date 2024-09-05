use std::{collections::BTreeMap, sync::Arc};

use chrono::{DateTime, Utc};
use types::entities::watched_user::WatchedUser;

use super::Data;

#[allow(clippy::unused_async)] // TODO: Remove this once the function is actually used
pub async fn act() -> BTreeMap<Arc<str>, Data> {
  let vec = get_from_db();

  // TODO: Get last post and use it
  let last_notified_watchers = DateTime::<Utc>::from_timestamp(0, 0).unwrap();

  vec
    .into_iter()
    .map(|u| {
      (
        Arc::from(u.did),
        Data {
          last_notified_watchers,
          watchers: u.watchers.0.into_iter().collect(),
        },
      )
    })
    .collect()
}

const fn get_from_db() -> Vec<WatchedUser> {
  // TODO
  vec![]
}
