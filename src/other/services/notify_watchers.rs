use std::sync::Arc;

use repositories::watched_user::WATCHED_USERS;
use tracing::{event, Level};
use types::entities::watched_user::Watcher;

pub async fn r#try(user_did: Arc<str>) {
  event!(Level::INFO, "Now notifying watchers of {user_did}.");
  let watched_users = WATCHED_USERS.get().await.read().await;
  let watchers = watched_users.get(&user_did).map(|w| &w.watchers);
  if let Some(watchers) = watchers {
    for u in watchers {
      #[allow(unused_variables)] // TODO: Actually implement this feature
      let Watcher { did, watch_replies } = u;
      tokio::spawn(notify_watcher(did.clone(), user_did.clone()));
    }
  }
  drop(watched_users);
}

#[allow(clippy::unused_async)] // TODO: Remove this once the function is actually used
async fn notify_watcher(watcher: Box<str>, user_did: Arc<str>) {
  // TODO: Check if should really use Box here
  event!(
    Level::DEBUG,
    "Successfully notified {user_did}'s watcher: {watcher}."
  );
}
