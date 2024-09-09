use std::collections::HashMap;

use anyhow::anyhow;
use atrium_api::chat::bsky::convo::defs::{
  MessageViewData, MessageViewSender, MessageViewSenderData,
};
use bsky::{send_message, Bsky};
use lazy_static::lazy_static;
use tokio::sync::RwLock;
use tracing::{event, Level};

use crate::commands;

lazy_static! {
  /// Pending messages to be processed.
  pub static ref PENDING_MESSAGES: RwLock<HashMap<String, MessageViewData>> =
    RwLock::new(HashMap::new());
}

/// Adds a message to the pending messages for later processing.
pub async fn add(convo_id: String, data: MessageViewData) {
  let agent_did = Bsky::get_agent_did().await;
  if *agent_did == *data.sender.data.did {
    event!(Level::DEBUG, "Ignoring message from self.");
    return;
  }

  PENDING_MESSAGES.write().await.insert(convo_id, data);
}

/// Processes a pending message by parsing the command then executing it.
/// If the command is successful, it will send the message back to the user.
///
/// # Errors
/// Propagates any errors that occur during the process of contacting the API.
pub async fn process(convo_id: String, data: MessageViewData) -> commands::Result<()> {
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
  let message = commands::parse(&text, facets)
    .await?
    .process(did)
    .await
    .map_err(|e| anyhow!(e))?;
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
