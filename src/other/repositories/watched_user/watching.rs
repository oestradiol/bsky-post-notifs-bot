use async_once::AsyncOnce;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
  collections::{HashMap, HashSet},
  hash::Hasher,
  sync::Arc,
};
use tokio::sync::RwLock;
use utils::Did;

use crate::Database;

lazy_static! {
  static ref STATE: AsyncOnce<Watching> = AsyncOnce::new(Watching::init());
}
pub struct Watching(RwLock<HashMap<Did, Watchers>>);
impl Watching {
  async fn init() -> Self {
    Self(RwLock::new(get_watching().await.unwrap_or_else(|e| {
      panic!("Failed to get initial state for watched users! Error: {e}")
    })))
  }

  /// Returns true if watched user is only now being watched
  pub async fn watch(watched_did: Did, watcher: Did, with_replies: bool) -> bool {
    let mut is_new = false;
    let new_watchers = || {
      is_new = true;
      Watchers::new()
    };
    let mut state_rw = STATE.get().await.0.write().await;
    let watchers = state_rw.entry(watched_did).or_insert_with(new_watchers);
    watchers.add(watcher, with_replies).await;
    drop(state_rw);

    is_new
  }

  /// Returns true if watched user is no longer being watched
  pub async fn unwatch(watched_did: &Did, watcher: Did) -> Option<bool> {
    let state = &STATE.get().await.0;
    let state_ro = state.read().await;
    let watchers = state_ro.get(watched_did)?;
    let is_no_longer = if watchers.remove(watcher).await {
      drop(state_ro);
      state.write().await.remove(watched_did);
      true
    } else {
      false
    };
    Some(is_no_longer)
  }

  pub async fn unwatch_all(watched_did: &Did) -> Option<HashSet<Watcher>> {
    let watchers = STATE.get().await.0.write().await.remove(watched_did)?;
    Some(watchers.clone().await)
  }

  pub async fn get_watchers(watched_did: &Did) -> Option<HashSet<Watcher>> {
    Some(
      STATE
        .get()
        .await
        .0
        .read()
        .await
        .get(watched_did)?
        .clone()
        .await,
    )
  }

  pub async fn get_watching() -> HashSet<Did> {
    STATE.get().await.0.read().await.keys().cloned().collect()
  }

  pub async fn get_watched_by(watcher_did: &Did) -> HashSet<Did> {
    let mut watched_by = HashSet::new();
    #[allow(clippy::significant_drop_in_scrutinee)] // Clippy bug, lol
    for (did, watchers) in STATE.get().await.0.read().await.iter() {
      if watchers.contains(watcher_did).await {
        watched_by.insert(did.clone());
      }
    }
    watched_by
  }

  pub async fn is_watched(watched_did: &Did) -> bool {
    STATE.get().await.0.read().await.contains_key(watched_did)
  }
}

struct Watchers(RwLock<HashSet<Watcher>>);
impl Watchers {
  fn new() -> Self {
    Self(RwLock::from(HashSet::new()))
  }

  async fn add(&self, watcher: Did, with_replies: bool) {
    self.0.write().await.insert(Watcher {
      did: watcher,
      watch_replies: with_replies,
    });
  }

  /// Returns true if Watchers is now empty
  async fn remove(&self, watcher: Did) -> bool {
    let mut watchers_mut = self.0.write().await;
    watchers_mut.remove(&Watcher {
      did: watcher,
      watch_replies: false,
    }); // watch_replies not used in the hash
    drop(watchers_mut);
    self.0.read().await.is_empty()
  }

  async fn clone(&self) -> HashSet<Watcher> {
    self.0.read().await.clone()
  }

  async fn contains(&self, watcher: &Did) -> bool {
    self.0.read().await.iter().any(|w| w.did == *watcher)
  }
}
impl From<HashSet<Watcher>> for Watchers {
  fn from(watchers: HashSet<Watcher>) -> Self {
    Self(RwLock::new(watchers))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct Watcher {
  #[serde(rename = "0")]
  pub did: Did,
  #[serde(rename = "1")]
  pub watch_replies: bool,
}
impl std::hash::Hash for Watcher {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.did.hash(state);
  }
}
impl PartialEq for Watcher {
  fn eq(&self, other: &Self) -> bool {
    self.did == other.did
  }
}

async fn get_watching() -> sqlx::Result<HashMap<Did, Watchers>> {
  let mut tx = Database::get_tx().await?;
  let watched_users = sqlx::query!(r#"SELECT * FROM "WatchedUser""#)
    .fetch_all(&mut *tx)
    .await?;
  tx.commit().await?;

  let mut watching = HashMap::new();
  for user in watched_users {
    let did = Arc::<str>::from(user.did);
    let watchers = serde_json::from_str::<HashSet<Watcher>>(&user.watchers)
      .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    watching.insert(did, Watchers::from(watchers));
  }
  Ok(watching)
}
