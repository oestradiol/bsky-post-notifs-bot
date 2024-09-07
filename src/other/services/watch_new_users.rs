use std::{collections::HashSet, hash::BuildHasher, sync::Arc};

use atrium_api::types::string::Did;
use repositories::watched_user;
use tracing::{event, Level};

use crate::{jobs, notify};

pub async fn act<S: BuildHasher + Send>(watcher: Did, watched_users: HashSet<Did, S>) {
  let watcher = Arc::<str>::from(String::from(watcher));
  for watched_did in watched_users
    .into_iter()
    .map(|w| Arc::<str>::from(String::from(w)))
  {
    // TODO: Actually implement with_replies feature: with_replies
    if watched_user::watch(watched_did.clone(), watcher.clone(), false).await {
      tokio::spawn(jobs::user_watcher::new(watched_did.clone()));
      tokio::spawn(async {
        notify::watched_user::now_watched(watched_did)
          .await
          .map_err(|e| {
            event!(
              Level::WARN,
              "(Notice) Error notifying newly watched user: {:?}",
              e
            );
          })
      });
    }
  }
}
