use repositories::watched_user;
use utils::Did;

use crate::notify;

/// Users opt-out by blocking the bot. So, we delete them from the db and notify their watchers.
pub async fn handle(watched_did: Did, is_block: bool) {
  let watchers = watched_user::unwatch_all(&watched_did, is_block).await;
  if !is_block {
    return;
  }

  if let Some(watchers) = watchers {
    notify::watcher::many(watched_did.clone(), Some(watchers), false).await;
    // No point in trying to notify the user if they've blocked the bot.
    // tokio::spawn(async move {
    //   notify::watched_user::no_longer(watched_did)
    //     .await
    //     .map_err(|e| event!(Level::WARN, "(Notice) Failed to notify user: {e}"))
    // });
  }
}
