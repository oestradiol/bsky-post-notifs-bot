use std::sync::Arc;

use atrium_api::types::string::{AtIdentifier, Did};
use repositories::watched_user;

use crate::resolve_handles;

use super::{Command, PinnedFut, Result};

#[derive(Debug)]
pub struct ListWatched;
impl Command for ListWatched {
  fn process(self: Box<Self>, sender_did: Did) -> PinnedFut<Result<String>> {
    Box::pin(async move {
      let watched: Vec<_> = watched_user::get_watched_by(&Arc::from(String::from(sender_did)))
        .await
        .into_iter()
        .map(|d| d.parse::<AtIdentifier>().unwrap())
        .collect();
      if watched.is_empty() {
        return Ok("You're not watching any users.".to_string());
      }

      let watched = resolve_handles::act(watched)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

      Ok(watched.into_iter().fold(
        "You're currently watching these users:".to_string(),
        |acc, handle| format!("{acc}\n- @{}", &*handle),
      ))
    })
  }
}
