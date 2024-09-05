use std::{collections::HashSet, sync::Arc};

use atrium_api::types::string::Did;
use repositories::watched_user::WATCHED_USERS;
use tracing::{event, Level};
use types::entities::watched_user::Watcher;

pub async fn act(listener: Did, to_unwatch: HashSet<Did>) {
  let watcher = Watcher {
    did: Box::from(String::from(listener)),
    watch_replies: false, // Can be any value, Watcher is simply here to be used in the hashing function
  };

  for user in to_unwatch
    .into_iter()
    .map(|w| Arc::<str>::from(String::from(w)))
  {
    let mut watched_users = WATCHED_USERS.get().await.write().await;
    if let Some(data) = watched_users.get_mut(&user) {
      data.watchers.remove(&watcher);
      if data.watchers.is_empty() {
        watched_users.remove(&user);
        drop(watched_users);
        // TODO: Notify user they're no longer being watched
        // TODO: Delete watched_user state from DB.
      } else {
        // TODO: Save watched_user state to DB.
      }
    } else {
      event!(
        Level::WARN,
        "(Notice) Weird... {user} was not found in WATCHED_USERS when it should have been. Was a command repeated?"
      );
      // TODO: At least try removing it from DB
    }
  }
}
