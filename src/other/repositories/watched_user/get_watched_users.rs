use std::{collections::BTreeMap, sync::Arc};

use sqlx::types::chrono::{DateTime, Utc};
use types::entities::watched_user::{WatchedUser, Watcher, Watchers};

use super::WatchedUserInfo;

pub async fn act() -> BTreeMap<Arc<str>, WatchedUserInfo> {
  let vec = get_from_db().await;

  // TODO: Get last post and use it
  let naive = DateTime::<Utc>::from_timestamp(0, 0).unwrap().naive_utc();

  vec
    .into_iter()
    .map(|u| {
      (
        Arc::from(u.did),
        WatchedUserInfo {
          last_peeked: naive,
          watchers: u.watchers.0,
        },
      )
    })
    .collect()
}

async fn get_from_db() -> Vec<WatchedUser> {
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
        did: me.clone(),
        watch_replies: true,
      }]),
    },
    WatchedUser {
      did: Box::from("odraws.bsky.social"),
      watchers: Watchers(vec![Watcher {
        did: me,
        watch_replies: true,
      }]),
    },
  ]
}
