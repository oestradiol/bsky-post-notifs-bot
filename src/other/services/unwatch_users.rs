use std::{collections::HashSet, hash::BuildHasher, sync::Arc};

use atrium_api::types::string::Did;
use repositories::watched_user;
use tracing::{event, Level};

use crate::notify;

pub async fn act<S: BuildHasher + Send>(watcher: Did, watched_users: HashSet<Did, S>) {
  let watcher = Arc::<str>::from(String::from(watcher));
  for watched_did in watched_users
    .into_iter()
    .map(|w| Arc::<str>::from(String::from(w)))
  {
    if let Some(true) = watched_user::unwatch(watched_did.clone(), watcher.clone()).await {
      tokio::spawn(async {
        notify::watched_user::no_longer(watched_did)
          .await
          .map_err(|e| {
            event!(
              Level::WARN,
              "(Notice) Error notifying unwatched watched user: {:?}",
              e
            );
          })
      });
    }
  }
}
