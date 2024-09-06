use std::{collections::HashSet, hash::BuildHasher, sync::Arc};

use atrium_api::{
  chat::bsky::convo::{defs::ConvoViewData, get_convo_for_members},
  types::{string::Did, Object},
};
use bsky::{get_user_convo, send_message};
use chrono::Utc;
use repositories::{watched_user::{self, Data, WATCHED_USERS}, Database};
use tracing::{event, Level};
use types::entities::watched_user::Watcher;

use crate::watch_users_for_posts::watch_user;

pub async fn act<S: BuildHasher + Send>(listener: Did, to_watch: HashSet<Did, S>) {
  let watcher = Watcher {
    did: Arc::from(String::from(listener)),
    watch_replies: false, // TODO: Stop cloning watcher once this feature is implemented
  };

  for user in to_watch
    .into_iter()
    .map(|w| Arc::<str>::from(String::from(w)))
  {
    let mut watched_users = WATCHED_USERS.get().await.write().await;
    if let Some(data) = watched_users.get_mut(&user) {
      data.watchers.insert(watcher.clone());
      drop(watched_users);
      // TODO: Likely to be in DB 
    } else {
      let mut watchers = HashSet::new();
      watchers.insert(watcher.clone());

      let last_notified_watchers = Utc::now();
      let data = Data {
        last_notified_watchers,
        watchers,
      };
      watched_users.insert(user.clone(), data);
      drop(watched_users);

      tokio::spawn(watch_user(user.clone(), last_notified_watchers));

      // TODO: Likely to not be in DB

      let _ = notify_watched(user.clone()).await.map_err(|e| {
        event!(
          Level::WARN,
          "(Notice) Error notifying watched user: {:?}",
          e
        );
      });
    }

    // TODO: Merge this with another occurrence and review code that i was too eepy to do properly yesterday: "create or update"
    let update_fut = async move {
      async move {
        let watched_users = WATCHED_USERS.get().await.read().await;
        if let Some(data) = watched_users.get(&user) {
          let mut tx = Database::get_tx().await?;
          let res = watched_user::update(&mut tx, user, &data.watchers).await?; // TODO: Handle None (User not in DB yet), check previous todos above
          drop(watched_users);
          tx.commit().await?;
        }
        Ok(())
      }.await.map_err(|e: anyhow::Error| event!(Level::ERROR, "(Notice) Failed to update watched user: {e}"))
    };
    tokio::spawn(update_fut);
  }
}

async fn notify_watched(user: Arc<str>) -> Result<(), anyhow::Error> {
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
