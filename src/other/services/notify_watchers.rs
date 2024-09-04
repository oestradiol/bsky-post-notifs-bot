use std::sync::Arc;

use repositories::watched_user::WATCHED_USERS;
use tokio::task::JoinSet;
use tracing::{event, Level};
use types::entities::watched_user::Watcher;

pub async fn r#try(user: Arc<str>) {
  let fut = async {
    let watcher = WATCHED_USERS.get().await.read().await;
    let watchers = &watcher.get(&user)?.watchers;
    let mut set = JoinSet::new();
    for u in watchers {
      let Watcher { did, watch_replies } = u;
      set.spawn(notify_watcher(did.clone(), user.clone()));
    }
    drop(watcher);

    while let Some(res) = set.join_next().await {
      let _ = res.map_err(|e| {
        event!(
          Level::ERROR,
          "Failed to Join notify_watchers_for_user: {:?}",
          e
        );
      });
    }

    Some(())
  };

  event!(Level::INFO, "Now notifying watchers of {user}.");
  let _ = fut.await;
}

#[allow(clippy::unused_async)]
async fn notify_watcher(watcher: Box<str>, user: Arc<str>) {
  event!(
    Level::DEBUG,
    "Successfully notified {user}'s watcher: {watcher}."
  );
  // TODO; Should handle error here!
}
