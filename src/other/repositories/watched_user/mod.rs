use std::collections::HashSet;

use crate::Database;
use tracing::{event, Level};

mod watching;
use utils::Did;
pub use watching::Watcher;
use watching::Watching;

mod db;

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

pub async fn unwatch_all(watched_did: &Did, sync_with_db: bool) -> Option<HashSet<Watcher>> {
  let watchers = Watching::unwatch_all(watched_did).await;
  if sync_with_db {
    tokio::spawn(handled_db_delete(watched_did.clone()));
  }
  watchers
}

pub async fn get_watchers(watched_did: &Did) -> Option<HashSet<Watcher>> {
  Watching::get_watchers(watched_did).await
}

pub async fn get_watching() -> HashSet<Did> {
  Watching::get_watching().await
}

pub async fn get_watched_by(watcher: &Did) -> HashSet<Did> {
  Watching::get_watched_by(watcher).await
}

pub async fn is_watched(watched_did: &Did) -> bool {
  Watching::is_watched(watched_did).await
}

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
    )
  });
}

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
