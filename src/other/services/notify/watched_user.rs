use atrium_api::{
  chat::bsky::convo::{defs::ConvoViewData, get_convo_for_members},
  types::Object,
};
use bsky::{get_user_convo, send_message};
use utils::Did;

pub async fn no_longer(watched_did: Did) -> Result<(), anyhow::Error> {
  #[allow(clippy::unwrap_used)] // Did from DB so always valid
  let watched_did = watched_did.parse().unwrap();

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

pub async fn now_watched(watched_did: Did) -> Result<(), anyhow::Error> {
  #[allow(clippy::unwrap_used)] // Did from DB so always valid
  let watched_did = watched_did.parse().unwrap();

  let get_convo_for_members::OutputData {
    convo: Object {
      data: ConvoViewData { id: convo_id, .. },
      ..
    },
    ..
  } = get_user_convo::act(watched_did).await?;
  send_message::act(
    convo_id,
    "\
Heads up! You're now being watched by someone. \
If you don't feel comfortable with this, \
you can opt-out by blocking this bot. \
If you have any questions, please read my bio!"
      .to_string(),
    false,
  )
  .await?;

  Ok(())
}
