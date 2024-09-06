use std::{collections::HashSet, hash::BuildHasher, sync::Arc};

use atrium_api::types::string::Did;
use repositories::{watched_user::{self, WATCHED_USERS}, Database};
use tracing::{event, Level};
use types::entities::watched_user::Watcher;

use crate::handle_user_unwatched;

pub async fn act<S: BuildHasher + Send>(listener: Did, to_unwatch: HashSet<Did, S>) {
  let watcher = Watcher {
    did: Arc::from(String::from(listener)),
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
        drop(watched_users);
        tokio::spawn(handle_user_unwatched::act(user.clone(), true));
      } else {
        // TODO: Merge this with another occurrence and review code that i was too eepy to do properly yesterday
        let update_fut = async move {
          async move {
            let watched_users = WATCHED_USERS.get().await.read().await;
            if let Some(data) = watched_users.get(&user) {
              let mut tx = Database::get_tx().await?;
              let res = watched_user::update(&mut tx, user, &data.watchers).await?; // TODO: Handle None (User not in DB yet)
              drop(watched_users);
              tx.commit().await?;
            }
            Ok(())
          }.await.map_err(|e: anyhow::Error| event!(Level::ERROR, "(Notice) Failed to update watched user: {e}"))
        };
        drop(watched_users);
        tokio::spawn(update_fut);
      }
    } else {
      event!(
        Level::WARN,
        "(Notice) Weird... {user} was not found in WATCHED_USERS when it should have been. Was a command repeated?"
      );

      let new_watcher = watcher.clone();
      let update_fut = async move {
        async move {
          let mut tx = Database::get_tx().await?;
          let res = watched_user::remove_if_found(&mut tx, user, new_watcher).await?;
          tx.commit().await?;
          Ok(res)
        }.await.map_err(|e: anyhow::Error| event!(Level::ERROR, "(Notice) Failed to update watched user: {e}"))
      };
      tokio::spawn(update_fut);
    }
  }
}
