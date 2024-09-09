use std::num::NonZeroU64;

use anyhow::anyhow;
use atrium_api::{
  chat::bsky::convo::{
    defs::{ConvoViewData, ConvoViewLastMessageRefs},
    get_messages::OutputMessagesItem,
  },
  types::Object,
};
use bsky::{get_messages, read_convo, Error};
use tokio::task::JoinSet;
use tracing::{event, Level};
use utils::handle_union;

use crate::pending_messages::add;

/// Method to handle unanswered conversations.
/// Receives a vector of conversations and handles them with the methods defined below.
pub async fn handle_many(dms: Vec<ConvoViewData>) {
  let mut set = JoinSet::new();
  for data in dms {
    set.spawn(async {
      handle(data)
        .await
        .map_err(|e| event!(Level::WARN, "(Notice) Failed to handle unanswered convo. Will try again on next iteration. Error: {e}"))
    });
  }
  drop(set.join_all().await);
}

/// Method to handle a single unanswered conversation. Tries to read the last message
/// message initially. If that fails, it then tries to fetch all of the unread messages
/// and handle them accordingly. Lastly, it tries to mark the conversation as read and
/// add the message to the pending messages for later processing.
///
/// # Errors
/// Propagates any errors that occur during the process of contacting the API.
async fn handle(convo: ConvoViewData) -> Result<(), Error<anyhow::Error>> {
  let ConvoViewData {
    id: convo_id,
    last_message,
    unread_count,
    ..
  } = convo;

  if unread_count == 1 {
    match last_message.and_then(handle_union) {
      Some(ConvoViewLastMessageRefs::MessageView(view)) => {
        let Object { data, .. } = *view;
        add(convo_id.clone(), data).await;
      }
      Some(ConvoViewLastMessageRefs::DeletedMessageView(_)) => {
        event!(Level::DEBUG, "Message has been unsent. Ignoring.");
      }
      None => {
        #[expect(clippy::unwrap_used)] // NonZeroU64, should never fail since 1
        fetch_and_handle_unread(convo_id.clone(), 1.try_into().unwrap()).await?;
      }
    }
  } else {
    #[expect(clippy::unwrap_used)] // NonZeroU64, never called with less than 1
    let as_non_zero = TryInto::<u64>::try_into(unread_count)
      .unwrap()
      .try_into()
      .unwrap();
    fetch_and_handle_unread(convo_id.clone(), as_non_zero).await?;
  };

  // Extremely unlikely to happen, unless their API
  // starts failing in the middle of handling these.
  // Still possible, however.
  let log = "\
    (Notice) Failed to mark convo as read. \
    Command will be added again. Possibly will be ignored by HashSet. \
    There exists, however, a high chance that it will be handled twice. \
    This possibly will cause the user to receive a duplicate response.";

  drop(
    read_convo::act(convo_id)
      .await
      .map_err(|_| event!(Level::WARN, log)),
  );

  Ok(())
}

/// Method to fetch and handle all unread messages in a conversation, in case
/// the last message was not able to be read.
///
/// # Errors
/// Propagates any errors that occur during the process of contacting the API.
async fn fetch_and_handle_unread(
  convo_id: String,
  unread_count: NonZeroU64,
) -> Result<(), Error<anyhow::Error>> {
  let unread_messages: Vec<_> = get_messages::act(convo_id.clone(), unread_count)
    .await
    .map_err(|e| anyhow!(e))?;
  for message in unread_messages {
    match message {
      OutputMessagesItem::ChatBskyConvoDefsMessageView(view) => {
        let Object { data, .. } = *view;
        add(convo_id.clone(), data).await;
      }
      OutputMessagesItem::ChatBskyConvoDefsDeletedMessageView(_) => {
        event!(Level::DEBUG, "Message has been unsent. Ignoring.");
      }
    }
  }
  Ok(())
}
