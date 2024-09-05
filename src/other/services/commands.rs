use std::collections::HashSet;

use atrium_api::{
  app::bsky::richtext::facet::{MainData, MainFeaturesItem, MentionData},
  types::{
    string::{AtIdentifier, Did},
    Object,
  },
};
use bsky::send_message;
use tracing::{event, Level};
use utils::handle_union;

#[derive(Debug)]
pub enum Command {
  Help,
  ListWatching,
  Watch(Vec<AtIdentifier>),
  Unwatch(Vec<AtIdentifier>),
  Invalid(String),
}

static AVAILABLE_COMMANDS: &str = "Available commands:
  - !watch @user_1 @user_2 (...)
  - !unwatch @user_1 @user_2 (...)
  - !list_watching
  - !help";

#[allow(unused_variables)] // TODO: Implement this command.
pub async fn issue_command(command: Command, convo_id: String, sender_did: Did) {
  drop(
    match command {
      Command::Help => send_message::act(convo_id, AVAILABLE_COMMANDS.to_string()),
      Command::Watch(user_dids) => {
        let user_dids = user_dids.into_iter().collect::<HashSet<_>>();
        // TODO: Actually watch for user.

        let message = user_dids // TODO: Fetch user handles from user_dids.
          .into_iter()
          .fold("Ok! Now watching users:".to_string(), |acc, handle| {
            format!("{}\n- {}", acc, handle.as_ref())
          });
        send_message::act(convo_id, message)
      }
      Command::Unwatch(user_dids) => {
        let user_dids = user_dids.into_iter().collect::<HashSet<_>>();
        // TODO: Actually unwatch for user.

        let message = user_dids // TODO: Fetch user handles from user_dids.
          .into_iter()
          .fold("Ok! Unwatched users:".to_string(), |acc, handle| {
            format!("{acc}\n- {}", handle.as_ref())
          });
        send_message::act(convo_id, message)
      }
      Command::ListWatching => {
        // TODO: Actually fetch list of watched users. Make sure to fetch handles!
        let user_handles = vec!["blabla", "blabla", "blabla"];

        let message = user_handles.into_iter().fold(
          "You're currently watching these users:".to_string(),
          |acc, handle| format!("{acc}\n- {handle}"),
        );
        send_message::act(convo_id, message)
      }
      Command::Invalid(message) => send_message::act(convo_id, message),
    }
    .await
    .map_err(|_| event!(Level::WARN, "Failed to send command message.")),
  );
}

static ZERO_MENTIONS: &str = "Please make sure to mention at least one user.";

pub fn parse_command(text: &str, facets: Option<Vec<Object<MainData>>>) -> Option<Command> {
  let text = text.trim();
  if !text.starts_with('!') {
    return None;
  }

  let mut parts = text.split_whitespace();
  let command = parts.next().unwrap_or_default(); // Will never be Default since checked above

  match command {
    "!help" => Some(Command::Help),
    "!watch" => facets.map_or_else(
      || Some(Command::Invalid(ZERO_MENTIONS.to_string())),
      |facets| {
        let mentions = extract_mentions(facets);
        if mentions.is_empty() {
          Some(Command::Invalid(ZERO_MENTIONS.to_string()))
        } else {
          Some(Command::Watch(mentions))
        }
      },
    ),
    "!unwatch" => facets.map_or_else(
      || Some(Command::Invalid(ZERO_MENTIONS.to_string())),
      |facets| {
        let mentions = extract_mentions(facets);
        if mentions.is_empty() {
          Some(Command::Invalid(ZERO_MENTIONS.to_string()))
        } else {
          Some(Command::Unwatch(mentions))
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
