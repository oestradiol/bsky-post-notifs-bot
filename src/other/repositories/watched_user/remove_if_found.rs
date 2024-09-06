use std::{collections::HashSet, sync::Arc};

use types::entities::watched_user::Watcher;

use crate::{AppTransaction, Loadable};

use super::update;

/// # Errors
/// 
/// Returns an error if the query or serialization fails.
pub async fn remove_if_found(tx: &mut AppTransaction, key: Arc<str>, watcher: Watcher) -> Loadable<()> {
  let key_as_str = &*key;
  let watchers_json = sqlx::query!(r#"SELECT watchers FROM "WatchedUser" WHERE did = $1"#, key_as_str)
    .fetch_optional(&mut **tx)
    .await?.map(|r| r.watchers);

  let watchers_json = match watchers_json {
    None => return Ok(None),
    Some(watchers_json) => watchers_json,
  };

  let mut watchers = serde_json::from_str::<HashSet<Watcher>>(&watchers_json).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
  watchers.remove(&watcher);
  update(tx, key, &watchers).await
}