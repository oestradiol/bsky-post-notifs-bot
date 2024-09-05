use std::hash::Hasher;

use serde::{Deserialize, Serialize};

type Did = Box<str>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedUser {
  #[serde(rename = "0")]
  pub did: Did, // PK
  #[serde(rename = "1")]
  pub watchers: Watchers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchers(pub Vec<Watcher>);

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
