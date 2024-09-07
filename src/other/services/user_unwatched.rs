use repositories::watched_user;
use tracing::{event, Level};
use utils::Did;

use crate::notify;

pub async fn handle(watched_did: Did, is_definitive: bool) {
  let watchers = watched_user::unwatch_all(&watched_did, is_definitive).await;
  if !is_definitive {
    return;
  }

  if let Some(watchers) = watchers {
    notify::watcher::many(watched_did.clone(), Some(watchers), false).await;
    tokio::spawn(async move {
      notify::watched_user::no_longer(watched_did)
        .await
        .map_err(|e| event!(Level::WARN, "(Notice) Failed to notify user: {e}"))
    });
  }
}
