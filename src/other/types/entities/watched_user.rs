use serde::{Deserialize, Serialize};

type Did = Box<str>;

#[derive(Debug, Clone)]
pub struct WatchedUser {
  pub did: Did, // PK
  pub watchers: Watchers,
}

#[derive(Debug, Clone)]
pub struct Watchers(pub Vec<Watcher>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watcher {
  #[serde(rename = "0")]
  pub did: Did,
  #[serde(rename = "1")]
  pub watch_replies: bool,
}
