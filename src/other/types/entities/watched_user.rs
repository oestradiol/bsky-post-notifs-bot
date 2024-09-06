use std::{collections::HashSet, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

type Did = Arc<str>;

#[derive(Debug, Clone)]
pub struct WatchedUser {
  pub did: Did, // PK
  pub watchers: HashSet<Watcher>,
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
