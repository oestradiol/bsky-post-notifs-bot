use std::{collections::HashSet, hash::RandomState};

use atrium_api::{
  chat::bsky::convo::{defs::ConvoViewData, get_convo_for_members},
  types::Object,
};
use bsky::{get_profile, get_user_convo, send_message};
use repositories::watched_user::{self, Watcher};
use tracing::{event, Level};
use utils::Did;

/// Notify the watchers of a watched user.
pub async fn many(
  watched_did: Did,
  watchers: Option<HashSet<Watcher, RandomState>>,
  is_post: bool,
) {
  event!(Level::DEBUG, "Now notifying watchers of {watched_did}.");
  let watchers = if watchers.is_some() {
    watchers
  } else {
    watched_user::get_watchers(&watched_did).await
  };

  if let Some(watchers) = watchers {
    for u in watchers {
      #[expect(unused_variables)] // TODO: Actually implement this feature
      let Watcher { did, watch_replies } = u;
      let watched_did = watched_did.clone();

      tokio::spawn(async move {
        act(did, watched_did, is_post).await.map_err(|e| {
          event!(Level::WARN, "(Notice) Failed to notify user: {e}");
        })
      });
    }
  }
}

/// Notify a single watcher of a watched user.
///
/// # Errors
/// Propagates any errors that occur during the process of contacting the API.
async fn act(watcher: Did, watched_did: Did, is_post: bool) -> Result<(), anyhow::Error> {
  #[expect(clippy::unwrap_used)] // Did from job so always valid
  let handle = get_profile::act(watched_did.parse().unwrap()).await?.handle;
  #[expect(clippy::unwrap_used)] // Did from DB so always valid
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
