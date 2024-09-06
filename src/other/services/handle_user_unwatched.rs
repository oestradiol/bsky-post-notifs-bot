use std::sync::Arc;

use atrium_api::{
  chat::bsky::convo::{defs::ConvoViewData, get_convo_for_members},
  types::Object,
};
use bsky::{get_user_convo, send_message};
use repositories::{watched_user::{self, WATCHED_USERS}, Database};
use tracing::{event, Level};

use crate::notify_watchers;

pub async fn act(watched_did: Arc<str>, is_definitive: bool) {
  let mut watched_users = WATCHED_USERS.get().await.write().await;
  let watched = watched_users.remove(&watched_did);
  drop(watched_users);
  if !is_definitive {
    return;
  }

  match watched {
    None => {
      event!(
        Level::WARN,
        "(Notice) Weird... {watched_did} was not found in WATCHED_USERS when it should have been. Was a command repeated?"
      );
    }
    Some(watched_user::Data { watchers, .. }) => {
      notify_watchers::r#try(watched_did.clone(), Some(watchers), false).await;
      let did = watched_did.clone();
      tokio::spawn(async move {
        notify_unwatched(did)
          .await
          .map_err(|e| event!(Level::WARN, "(Notice) Failed to notify user: {e}"))
      });
    }
  }

  let delete_fut = async move {
    async move {
      let mut tx = Database::get_tx().await?;
      let res = watched_user::delete(&mut tx, watched_did).await?;
      tx.commit().await?;
      Ok(res)
    }.await.map_err(|e: anyhow::Error| event!(Level::ERROR, "(Notice) Failed to delete watched user: {e}"))
  };
  tokio::spawn(delete_fut);
}

async fn notify_unwatched(user: Arc<str>) -> Result<(), anyhow::Error> {
  #[allow(clippy::unwrap_used)] // Did from DB so always valid
  let watched_did = user.parse().unwrap();

  let get_convo_for_members::OutputData {
    convo: Object {
      data: ConvoViewData { id: convo_id, .. },
      ..
    },
    ..
  } = get_user_convo::act(watched_did).await?;
  send_message::act(
    convo_id,
    "(Notice) You're no longer being watched by anyone.".to_string(),
    false,
  )
  .await?;

  Ok(())
}
