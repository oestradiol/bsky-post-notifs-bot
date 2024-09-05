use std::collections::HashSet;

use atrium_api::{
  app::bsky::richtext::facet::{MainData, MainFeaturesItem, MentionData},
  types::{
    string::{AtIdentifier, Did},
    Object,
  },
};
use utils::handle_union;

#[derive(Debug)]
pub enum Command {
  Help,
  ListWatching,
  Watch(HashSet<AtIdentifier>),
  Unwatch(HashSet<AtIdentifier>),
  Invalid(String),
}

static AVAILABLE_COMMANDS: &str = "Available commands:
  - `!watch @user_1 @user_2 (...)`
  - `!unwatch @user_1 @user_2 (...)`
  - `!list_watching`
  - `!help`";

#[allow(unused_variables)] // TODO: Implement this command.
#[allow(clippy::unused_async)]
pub async fn issue_command(command: Command, sender_did: Did) -> String {
  match command {
    Command::Help => AVAILABLE_COMMANDS.to_string(),
    Command::Watch(user_dids) => {
      // TODO: Actually watch for user.

      user_dids // TODO: Fetch user handles from user_dids.
        .into_iter()
        .fold("Ok! Now watching users:".to_string(), |acc, handle| {
          format!("{}\n- {}", acc, handle.as_ref())
        })
    }
    Command::Unwatch(user_dids) => {
      // TODO: Actually unwatch for user.

      user_dids // TODO: Fetch user handles from user_dids.
        .into_iter()
        .fold("Ok! Unwatched users:".to_string(), |acc, handle| {
          format!("{acc}\n- {}", handle.as_ref())
        })
    }
    Command::ListWatching => {
      // TODO: Actually fetch list of watched users. Make sure to fetch handles!
      let user_handles = vec!["blabla", "blabla", "blabla"];

      user_handles.into_iter().fold(
        "You're currently watching these users:".to_string(),
        |acc, handle| format!("{acc}\n- {handle}"),
      )
    }
    Command::Invalid(message) => message,
  }
}

static ZERO_MENTIONS: &str = "Please make sure to mention at least one user.";

pub fn parse_command(text: &str, facets: Option<Vec<Object<MainData>>>) -> Option<Command> {
  let text = text.trim();
  if !text.starts_with('!') {
    return None;
  }

  let mut parts = text.split_whitespace();
  #[allow(clippy::unwrap_used)] // Checked above
  let command = parts.next().unwrap();

  match command {
    "!help" => Some(Command::Help),
    "!watch" => facets.map_or_else(
      || Some(Command::Invalid(ZERO_MENTIONS.to_string())),
      |facets| {
        let user_dids = extract_mentions(facets);
        let user_dids = user_dids.into_iter().collect::<HashSet<_>>();
        if user_dids.is_empty() {
          Some(Command::Invalid(ZERO_MENTIONS.to_string()))
        } else {
          Some(Command::Watch(user_dids))
        }
      },
    ),
    "!unwatch" => facets.map_or_else(
      || Some(Command::Invalid(ZERO_MENTIONS.to_string())),
      |facets| {
        let user_dids = extract_mentions(facets);
        let user_dids = user_dids.into_iter().collect::<HashSet<_>>();
        if user_dids.is_empty() {
          Some(Command::Invalid(ZERO_MENTIONS.to_string()))
        } else {
          Some(Command::Unwatch(user_dids))
        }
      },
    ),
    "!list_watching" => Some(Command::ListWatching),
    _ => Some(Command::Invalid(
      "Invalid command. You can get a list of available commands with `!help`.".to_string(),
    )),
  }
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
