use std::collections::HashSet;

use tracing::{event, Level};
use utils::Did;

use crate::{watched_user::Watcher, AppTransaction, Loadable};

/// # Errors
///
/// Returns an error if the query or serialization fails.
pub async fn create(
  tx: &mut AppTransaction,
  watched_did: &Did,
  watcher: Did,
  with_replies: bool,
) -> Loadable<()> {
  let watchers = serde_json::to_string(&HashSet::from([Watcher {
    did: watcher,
    watch_replies: with_replies,
  }]))
  .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

  let did = &**watched_did;
  let rows = sqlx::query!(
    r#"INSERT INTO "WatchedUser" (did, watchers) VALUES ($1, $2)"#,
    did,
    watchers
  )
  .execute(&mut **tx)
  .await?
  .rows_affected();

  // TODO: check, this should be one!!
  event!(Level::WARN, "rows affected: {}", rows);

  Ok(if rows > 0 { Some(()) } else { None })
}
