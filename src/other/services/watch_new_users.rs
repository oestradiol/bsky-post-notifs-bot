use std::{collections::HashSet, sync::Arc};

use atrium_api::types::string::Did;
use chrono::{DateTime, Utc};
use repositories::watched_user::{Data, WATCHED_USERS};
use types::entities::watched_user::Watcher;

use crate::watch_users_for_posts::watch_user;

pub async fn act(listener: Did, to_watch: HashSet<Did>) {
  let watcher = Watcher {
    did: Box::from(String::from(listener)),
    watch_replies: false, // TODO: Stop cloning watcher once this feature is implemented
  };

  for user in to_watch
    .into_iter()
    .map(|w| Arc::<str>::from(String::from(w)))
  {
    let mut watched_users = WATCHED_USERS.get().await.write().await;
    if let Some(data) = watched_users.get_mut(&user) {
      data.watchers.insert(watcher.clone());
    } else {
      let mut watchers = HashSet::new();
      watchers.insert(watcher.clone());

      let last_notified_watchers = DateTime::<Utc>::from_timestamp(0, 0).unwrap(); // TODO: Get
      let data = Data {
        last_notified_watchers,
        watchers,
      };
      watched_users.insert(user.clone(), data);
      drop(watched_users);

      tokio::spawn(watch_user(user, last_notified_watchers));

      // TODO: Notify user they're now being watched by someone (keep anonymous)
    }
    // TODO: Also save watched_user state to DB.
  }
}
