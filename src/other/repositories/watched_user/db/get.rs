use std::collections::HashSet;

use utils::Did;

use crate::{watched_user::watching::Watcher, AppTransaction};

/// Returns a set of all watchers of a user.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn get(
  tx: &mut AppTransaction,
  watched_did: &Did,
) -> Result<HashSet<Watcher>, sqlx::Error> {
  let did = &**watched_did;
  let watchers = sqlx::query!(r#"SELECT watchers FROM "WatchedUser" WHERE did = $1"#, did)
    .fetch_one(&mut **tx)
    .await?
    .watchers;

  serde_json::from_str::<HashSet<Watcher>>(&watchers).map_err(|e| sqlx::Error::Decode(Box::new(e)))
}
