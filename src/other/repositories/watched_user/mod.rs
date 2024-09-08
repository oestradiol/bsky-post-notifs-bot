//! This module contains all the re-exported interfaces for manipulating the
//! memory repository and database of watched users.

use std::collections::HashSet;

use crate::Database;
use tracing::{event, Level};

mod watching;
use utils::Did;
pub use watching::Watcher;
use watching::Watching;

mod db;

/// Watches a user.
/// Returns true if the user is only now being watched (first watcher).
pub async fn watch(watched_did: Did, watcher: Did, with_replies: bool) -> bool {
  if Watching::watch(watched_did.clone(), watcher.clone(), with_replies).await {
    tokio::spawn(handled_db_create(watched_did, watcher, with_replies));
    true
  } else {
    tokio::spawn(handled_db_insert_watcher(
      watched_did,
      watcher,
      with_replies,
    ));
    false
  }
}

/// Unwatches a user.
/// Returns `Some(true)` if the user is no longer being watched (last watcher).
/// Returns `Some(false)` if the user is still being watched by other users.
/// Returns `None` if the user is not even being watched to begin with.
pub async fn unwatch(watched_did: Did, watcher: Did) -> Option<bool> {
  match Watching::unwatch(&watched_did, watcher.clone()).await {
    Some(true) => {
      tokio::spawn(handled_db_delete(watched_did));
      Some(true)
    }
    Some(false) => {
      tokio::spawn(handled_db_remove_watcher(watched_did, watcher));
      Some(false)
    }
    None => None,
  }
}

/// Unwatches a user from all watchers.
/// Returns a `Some` of a set of all watchers that were watching the user.
/// Returns `None` if the user is not even being watched to begin with.
pub async fn unwatch_all(watched_did: &Did, sync_with_db: bool) -> Option<HashSet<Watcher>> {
  let watchers = Watching::unwatch_all(watched_did).await;
  if sync_with_db {
    tokio::spawn(handled_db_delete(watched_did.clone()));
  }
  watchers
}

/// Returns a `Some` of set of all watchers of a user.
/// Returns `None` if the user is not even being watched to begin with.
pub async fn get_watchers(watched_did: &Did) -> Option<HashSet<Watcher>> {
  Watching::get_watchers(watched_did).await
}

/// Returns a set of all watched users.
pub async fn get_watching() -> HashSet<Did> {
  Watching::get_watching().await
}

/// Returns a set of all users watched by a user.
pub async fn get_watched_by(watcher: &Did) -> HashSet<Did> {
  Watching::get_watched_by(watcher).await
}

/// Returns true if a user is being watched.
pub async fn is_watched(watched_did: &Did) -> bool {
  Watching::is_watched(watched_did).await
}

/// Auxiliary function to handle database create operations.
/// Used when a user is being watched by a new watcher, and was not being watched
/// beforehand.
async fn handled_db_create(watched_did: Did, watcher: Did, with_replies: bool) {
  let _ = async move {
    let mut tx = Database::get_tx().await?;
    let res = db::create(&mut tx, &watched_did, watcher, with_replies).await;
    tx.commit().await?;
    res
  }
  .await
  .map_err(|e| event!(Level::WARN, "Failed to save watched user to Sqlite: {e}"));
}

/// Auxiliary function to handle database delete operations.
/// Used when a user is no longer being watched by any watcher.
async fn handled_db_delete(watched_did: Did) {
  let _ = async move {
    let mut tx = Database::get_tx().await?;
    let res = db::delete(&mut tx, &watched_did).await;
    tx.commit().await?;
    res
  }
  .await
  .map_err(|e| {
    event!(
      Level::WARN,
      "Failed to delete watched user from Sqlite: {e}"
    );
  });
}

/// Auxiliary function to handle database insert operations for a watcher.
/// Used when a user is being watched by a new watcher, but was already being watched
/// by other watchers.
async fn handled_db_insert_watcher(watched_did: Did, watcher: Did, with_replies: bool) {
  let _ = async move {
    let mut tx = Database::get_tx().await?;
    let res = db::insert_watcher(&mut tx, &watched_did, watcher, with_replies).await;
    tx.commit().await?;
    res
  }
  .await
  .map_err(|e| event!(Level::WARN, "Failed to insert new watcher in Sqlite: {e}"));
}

/// Auxiliary function to handle database remove operations for a watcher.
/// Used when a user is no longer being watched by a watcher, but there are still
/// other watchers.
async fn handled_db_remove_watcher(watched_did: Did, watcher: Did) {
  let _ = async move {
    let mut tx = Database::get_tx().await?;
    let res = db::remove_watcher(&mut tx, &watched_did, watcher).await;
    tx.commit().await?;
    res
  }
  .await
  .map_err(|e| event!(Level::WARN, "Failed to remove watcher from Sqlite: {e}"));
}
