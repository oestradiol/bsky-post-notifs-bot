use std::{collections::BTreeMap, sync::Arc};

use chrono::{DateTime, Utc};
use types::entities::watched_user::{WatchedUser, Watcher, Watchers};

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
          watchers: u.watchers.0,
        },
      )
    })
    .collect()
}

fn get_from_db() -> Vec<WatchedUser> {
  // TODO
  let me: Box<str> = Box::from("elynn.bsky.social");
  vec![
    WatchedUser {
      did: me.clone(),
      watchers: Watchers(vec![Watcher {
        did: me.clone(),
        watch_replies: true,
      }]),
    },
    WatchedUser {
      did: Box::from("logemi.xyz"),
      watchers: Watchers(vec![Watcher {
        did: me.clone(),
        watch_replies: true,
      }]),
    },
    WatchedUser {
      did: Box::from("felina.fish"),
      watchers: Watchers(vec![Watcher {
        did: me,
        watch_replies: true,
      }]),
    },
  ]
}
