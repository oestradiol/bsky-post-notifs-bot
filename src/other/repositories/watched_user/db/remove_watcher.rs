use utils::Did;

use crate::{watched_user::watching::Watcher, AppTransaction, Loadable};

use super::get;

/// # Errors
///
/// Returns an error if the queries or serialization fails.
pub async fn remove_watcher(
  tx: &mut AppTransaction,
  watched_did: &Did,
  watcher: Did,
) -> Loadable<()> {
  let mut watchers = get(tx, watched_did).await?;

  if watchers.remove(&Watcher {
    did: watcher,
    watch_replies: false, // watch_replies not used in the hash
  }) {
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

    return Ok(if rows > 0 { Some(()) } else { None });
  }

  Ok(Some(()))
}
