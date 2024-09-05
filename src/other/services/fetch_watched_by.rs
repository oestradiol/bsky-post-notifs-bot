use std::sync::Arc;

use atrium_api::types::string::Did;
use repositories::watched_user::WATCHED_USERS;

pub async fn act(sender_id: Did) -> Vec<Arc<str>> {
  WATCHED_USERS
    .get()
    .await
    .read()
    .await
    .iter()
    .fold(Vec::new(), |mut acc, (user, data)| {
      if data
        .watchers
        .iter()
        .any(|w| w.did.as_ref() == sender_id.as_ref())
      {
        acc.push(user.clone());
      }
      acc
    })
}
