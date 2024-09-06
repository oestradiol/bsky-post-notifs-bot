use std::collections::HashSet;

use atrium_api::{
  app::bsky::richtext::facet::{MainData, MainFeaturesItem, MentionData},
  types::{
    string::{AtIdentifier, Did, Handle},
    Object,
  },
};
use bsky::get_profiles;
use tracing::{event, Level};
use utils::handle_union;

use crate::{
  fetch_watched_by, resolve_dids_and_handles, resolve_handles, unwatch_users, watch_new_users,
};

#[derive(Debug)]
pub enum Command {
  Help,
  ListWatched,
  Watch(HashSet<Did>, HashSet<Handle>),
  Unwatch(HashSet<Did>, HashSet<Handle>),
  Invalid(String),
}

static AVAILABLE_COMMANDS: &str = "Available commands:
  - `!watch @user_1 @user_2 (...)`
  - `!unwatch @user_1 @user_2 (...)`
  - `!list_watched`
  - `!help`";

/// # Errors
///
/// `ListWatched` might fail to resolve handles
pub async fn issue_command(
  command: Command,
  sender_did: Did,
) -> Result<String, bsky::Error<anyhow::Error>> {
  let message = match command {
    Command::Help => AVAILABLE_COMMANDS.to_string(),
    Command::Watch(dids, handles) => {
      watch_new_users::act(sender_did, dids).await;

      handles
        .into_iter()
        .fold("Now watching users:".to_string(), |acc, handle| {
          format!("{}\n- @{}", acc, handle.as_ref())
        })
    }
    Command::Unwatch(dids, handles) => {
      unwatch_users::act(sender_did, dids).await;

      handles
        .into_iter()
        .fold("Unwatched users:".to_string(), |acc, handle| {
          format!("{acc}\n- @{}", handle.as_ref())
        })
    }
    Command::ListWatched => {
      #[allow(clippy::unwrap_used)] // Guaranteed to be valid since it's coming from the database
      let watched: Vec<_> = fetch_watched_by::act(sender_did)
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

      watched.into_iter().fold(
        "You're currently watching these users:".to_string(),
        |acc, handle| format!("{acc}\n- @{}", &*handle),
      )
    }
    Command::Invalid(message) => message,
  };
  Ok(message)
}

static ZERO_MENTIONS: &str = "Please make sure to mention at least one user.";

/// # Errors
///
/// Watch and Unwatch might fail to resolve handles and dids
pub async fn parse_command(
  text: &str,
  facets: Option<Vec<Object<MainData>>>,
) -> Result<Option<Command>, bsky::Error<get_profiles::Error>> {
  let text = text.trim();
  if !text.starts_with('!') {
    event!(
      Level::DEBUG,
      "Message does not start with a command prefix: {text}"
    );
    return Ok(None);
  }

  let mut parts = text.split_whitespace();
  #[allow(clippy::unwrap_used)] // Checked above
  let command = parts.next().unwrap().to_lowercase();

  let res = match command.as_str() {
    "!help" => Some(Command::Help),
    "!watch" => match facets {
      None => Some(Command::Invalid(ZERO_MENTIONS.to_string())),
      Some(facets) => {
        let at_ids = extract_mentions(facets);
        if at_ids.is_empty() {
          Some(Command::Invalid(ZERO_MENTIONS.to_string()))
        } else {
          let (dids, handles) = resolve_dids_and_handles::act(at_ids).await?;
          Some(Command::Watch(dids, handles))
        }
      }
    },
    "!unwatch" => match facets {
      None => Some(Command::Invalid(ZERO_MENTIONS.to_string())),
      Some(facets) => {
        let at_ids = extract_mentions(facets);
        if at_ids.is_empty() {
          Some(Command::Invalid(ZERO_MENTIONS.to_string()))
        } else {
          let (dids, handles) = resolve_dids_and_handles::act(at_ids).await?;
          Some(Command::Unwatch(dids, handles))
        }
      }
    },
    "!list_watched" => Some(Command::ListWatched),
    _ => Some(Command::Invalid(
      "Invalid command. You can get a list of available commands with `!help`.".to_string(),
    )),
  };
  Ok(res)
}

fn extract_mentions(facets: Vec<Object<MainData>>) -> Vec<AtIdentifier> {
  facets
    .into_iter()
    .flat_map(
      |Object {
         data: MainData { features, .. },
         ..
       }| features,
    )
    .filter_map(handle_union)
    .filter_map(|facet| match facet {
      MainFeaturesItem::Mention(mention) => {
        let Object {
          data: MentionData { did },
          ..
        } = *mention;
        Some(did)
      }
      _ => None,
    })
    .map(AtIdentifier::from)
    .collect()
}
