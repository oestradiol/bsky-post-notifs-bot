use std::{collections::HashSet, hash::RandomState, sync::Arc};

use atrium_api::{
  chat::bsky::convo::{defs::ConvoViewData, get_convo_for_members},
  types::Object,
};
use bsky::{get_profile, get_user_convo, send_message};
use repositories::watched_user::WATCHED_USERS;
use tracing::{event, Level};
use types::entities::watched_user::Watcher;

pub async fn r#try(watched_did: Arc<str>, watchers: Option<HashSet<Watcher, RandomState>>, is_post: bool) {
  event!(Level::INFO, "Now notifying watchers of {watched_did}.");
  let watched_users = WATCHED_USERS.get().await.read().await;

  let watchers = watchers.or_else(|| watched_users.get(&watched_did).map(|w| w.watchers.clone()));
  if let Some(watchers) = watchers {
    for u in watchers {
      #[allow(unused_variables)] // TODO: Actually implement this feature
      let Watcher { did, watch_replies } = u;
      let watched_did = watched_did.clone();

      tokio::spawn(async move {
        notify_watcher(did, watched_did, is_post)
          .await
          .map_err(|e| {
            event!(Level::WARN, "(Notice) Failed to notify user: {e}");
          })
      });
    }
  }
}

async fn notify_watcher(
  watcher: Arc<str>,
  watched_did: Arc<str>,
  is_post: bool,
) -> Result<(), anyhow::Error> {
  #[allow(clippy::unwrap_used)] // Did from job so always valid
  let handle = get_profile::act(watched_did.parse().unwrap()).await?.handle;
  #[allow(clippy::unwrap_used)] // Did from DB so always valid
  let get_convo_for_members::OutputData {
    convo: Object {
      data: ConvoViewData { id: convo_id, .. },
      ..
    },
    ..
  } = get_user_convo::act(watcher.parse().unwrap()).await?;

  let message = if is_post {
    format!(
      "Hey! Just wanted to let you know that @{} has posted something new. You might want to check it out!",
      &*handle
    )
  } else {
    format!(
      "(Notice) @{} has opted-out of being watched... You will no longer receive notifications! Lame...",
      &*handle
    )
  };

  send_message::act(convo_id, message, true).await?;

  event!(
    Level::DEBUG,
    "Successfully notified {watched_did}'s watcher: {watcher}."
  );

  Ok(())
}
