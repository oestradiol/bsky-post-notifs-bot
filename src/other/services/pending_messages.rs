use std::collections::HashMap;

use anyhow::anyhow;
use atrium_api::chat::bsky::convo::defs::{
  MessageViewData, MessageViewSender, MessageViewSenderData,
};
use bsky::{send_message, Bsky};
use lazy_static::lazy_static;
use tokio::sync::RwLock;
use tracing::{event, Level};

use crate::commands::{issue_command, parse_command};

lazy_static! {
  pub static ref PENDING_MESSAGES: RwLock<HashMap<String, MessageViewData>> =
    RwLock::new(HashMap::new());
}

pub async fn add_pending(convo_id: String, data: MessageViewData) {
  let agent_did = Bsky::get_agent_did().await;
  if *agent_did == *data.sender.data.did {
    event!(Level::DEBUG, "Ignoring message from self.");
    return;
  }

  PENDING_MESSAGES.write().await.insert(convo_id, data);
}

pub async fn handle_pending(convo_id: String, data: MessageViewData) -> Result<(), anyhow::Error> {
  let MessageViewData {
    facets,
    text,
    sender: MessageViewSender {
      data: MessageViewSenderData { did },
      ..
    },
    ..
  } = data;

  event!(Level::DEBUG, "Handling message from user {}: {text}", &*did);
  let command = parse_command(&text, facets).await?;
  let message = if let Some(command) = command {
    let log = format!("Issued command for user {}: {:?}.", &*did, command);
    let res = issue_command(command, did).await.map_err(|e| anyhow!(e))?;
    event!(Level::DEBUG, log);
    res
  } else {
    "\
If you're trying to issue a command, please use the command prefix:
  - `!<command>`

You can get a list of available commands with:
  - `!help`
"
    .to_string()
  };
  drop(
    send_message::act(convo_id, message, false)
      .await
      .map_err(|e| {
        event!(
      Level::WARN,
      "(Notice) Failed to send command message. Command completed successfully, however. Error: {e}"
    );
      }),
  );
  Ok(())
}
