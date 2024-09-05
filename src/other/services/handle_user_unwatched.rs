use std::sync::Arc;

use repositories::watched_user::{self, WATCHED_USERS};
use tracing::{event, Level};

pub async fn act(user_did: Arc<str>, is_opt_out: bool) {
  let mut watched_users = WATCHED_USERS.get().await.write().await;
  let watched = watched_users.remove(&user_did);
  drop(watched_users);
  if !is_opt_out {
    return;
  }

  match watched {
    #[allow(unused_variables)] // TODO: Delete from DB and notify watchers.
    Some(watched_user::Data { watchers, .. }) => {
      event!(
        Level::INFO,
        "Now notifying watchers of {user_did} that they have been unwatched."
      );
    }
    None => {
      event!(
        Level::WARN,
        "Weird... {user_did} was not found in WATCHED_USERS when it should have been."
      );
    }
  }
}
