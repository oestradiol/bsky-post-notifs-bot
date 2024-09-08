use utils::Did;

use crate::{watched_user::watching::Watcher, AppTransaction, Loadable};

use super::get;

/// Inserts a watcher to a watched user.
///
/// # Errors
///
/// Returns an error if the query or serialization fails.
pub async fn insert_watcher(
  tx: &mut AppTransaction,
  watched_did: &Did,
  watcher: Did,
  with_replies: bool,
) -> Loadable<()> {
  let mut watchers = get(tx, watched_did).await?;
  let watcher = Watcher {
    did: watcher,
    watch_replies: with_replies,
  };
  watchers.insert(watcher);
  let watchers_string =
    serde_json::to_string(&watchers).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

  let did = &**watched_did;
  let rows = sqlx::query!(
    r#"UPDATE "WatchedUser" SET watchers = $1 WHERE did = $2"#,
    watchers_string,
    did
  )
  .execute(&mut **tx)
  .await?
  .rows_affected();

  Ok(if rows > 0 { Some(()) } else { None })
}
