use std::{num::NonZeroU64, sync::Arc};

use atrium_api::{
  chat::bsky::convo::{
    defs::{
      ConvoView, ConvoViewData, ConvoViewLastMessageRefs, MessageViewData, MessageViewSender,
      MessageViewSenderData,
    },
    get_messages::OutputMessagesItem,
  },
  types::Object,
};
use bsky::{fetch_messages, send_message, Error};
use tracing::{event, Level};
use utils::handle_union;

use crate::commands::{issue_command, parse_command};

pub async fn act(dms: Vec<ConvoView>) {
  for Object {
    data: ConvoViewData {
      id,
      last_message,
      unread_count,
      ..
    },
    ..
  } in dms
  {
    let id = Arc::from(id);
    let res = if unread_count == 1 {
      if let Some(refs) = last_message.and_then(handle_union) {
        match refs {
          ConvoViewLastMessageRefs::MessageView(view) => handle_view(view, id).await,
          ConvoViewLastMessageRefs::DeletedMessageView(_) => {
            event!(Level::DEBUG, "Message has been unsent. Ignoring.");
          }
        }
        Ok(())
      } else {
        #[allow(clippy::unwrap_used)] // NonZeroU64, should never fail since 1
        fetch_and_handle_unread(id, 1.try_into().unwrap()).await
      }
    } else {
      #[allow(clippy::unwrap_used)]
      // NonZeroU64, this method should never be called with less than 1
      let as_non_zero = TryInto::<u64>::try_into(unread_count)
        .unwrap()
        .try_into()
        .unwrap();
      fetch_and_handle_unread(id, as_non_zero).await
    };
    let _ = res.map_err(|e| {
      event!(
        Level::WARN,
        "Error while fetching messages to handle: {e:?}"
      )
    });
  }
}

async fn fetch_and_handle_unread(
  convo_id: Arc<str>,
  unread_count: NonZeroU64,
) -> Result<(), Error<fetch_messages::Error>> {
  let unread_messages: Vec<_> = fetch_messages::act(convo_id.clone(), unread_count)
    .await?
    .into_iter()
    .filter_map(handle_union)
    .collect();
  for message in unread_messages {
    match message {
      OutputMessagesItem::ChatBskyConvoDefsMessageView(view) => {
        handle_view(view, convo_id.clone()).await
      }
      OutputMessagesItem::ChatBskyConvoDefsDeletedMessageView(_) => {
        event!(Level::DEBUG, "Message has been unsent. Ignoring.");
      }
    }
  }
  Ok(())
}

async fn handle_view(view: Box<Object<MessageViewData>>, convo_id: Arc<str>) {
  let Object {
    data:
      MessageViewData {
        facets,
        text,
        sender:
          MessageViewSender {
            data: MessageViewSenderData { did },
            ..
          },
        ..
      },
    ..
  } = *view;
  let command = parse_command(&text, facets);
  if let Some(command) = command {
    event!(Level::DEBUG, "Issuing command: {command:?}");
    issue_command(command, convo_id, did).await;
  } else {
    let message =
      "If you're trying to issue a command, please use the command prefix: `!<command>`. \
       You can get a list of available commands with `!help`."
        .to_string();
    drop(
      send_message::act(convo_id, message)
        .await
        .map_err(|_| event!(Level::WARN, "Failed to send 'confused user' message.")),
    );
  }
}
