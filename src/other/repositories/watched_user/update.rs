use std::{collections::HashSet, hash::BuildHasher, sync::Arc};

use types::entities::watched_user::Watcher;

use crate::{AppTransaction, Loadable};

/// # Errors
/// 
/// Returns an error if the query or serialization fails.
pub async fn update<S: BuildHasher + Sync>(tx: &mut AppTransaction, did: Arc<str>, watchers: &HashSet<Watcher, S>) -> Loadable<()> {
  let did = &*did;
  let watchers = serde_json::to_string(watchers).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
  let rows = sqlx::query!(r#"UPDATE "WatchedUser" SET watchers = $1 WHERE did = $2"#, watchers, did)
    .execute(&mut **tx)
    .await?
    .rows_affected();

  Ok(if rows > 0 { Some(()) } else { None })
}