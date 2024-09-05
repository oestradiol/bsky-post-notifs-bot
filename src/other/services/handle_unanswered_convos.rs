use std::num::NonZeroU64;

use anyhow::anyhow;
use atrium_api::{
  chat::bsky::convo::{
    defs::{ConvoView, ConvoViewData, ConvoViewLastMessageRefs},
    get_messages::OutputMessagesItem,
  },
  types::Object,
};
use bsky::{fetch_messages, read_convo, Error};
use tokio::task::JoinSet;
use tracing::{event, Level};
use utils::handle_union;

use crate::pending_messages::add_pending;

pub async fn act(dms: Vec<ConvoView>) {
  let mut set = JoinSet::new();
  for Object { data, .. } in dms {
    set.spawn(async {
      handle_unanswered_convo(data)
        .await
        .map_err(|e| event!(Level::WARN, "(Notice) Failed to handle unanswered convo. Will try again on next iteration. Error: {e}"))
    });
  }
  drop(set.join_all().await);
}

async fn handle_unanswered_convo(convo: ConvoViewData) -> Result<(), Error<anyhow::Error>> {
  let ConvoViewData {
    id,
    last_message,
    unread_count,
    ..
  } = convo;

  if unread_count == 1 {
    if let Some(refs) = last_message.and_then(handle_union) {
      match refs {
        ConvoViewLastMessageRefs::MessageView(view) => {
          let Object { data, .. } = *view;
          add_pending(id.clone(), data).await;
        }
        ConvoViewLastMessageRefs::DeletedMessageView(_) => {
          event!(Level::DEBUG, "Message has been unsent. Ignoring.");
        }
      }
    } else {
      #[allow(clippy::unwrap_used)] // NonZeroU64, should never fail since 1
      fetch_and_handle_unread(id.clone(), 1.try_into().unwrap()).await?;
    }
  } else {
    #[allow(clippy::unwrap_used)]
    // NonZeroU64, handle_unanswered_convo should never be called with less than 1
    let as_non_zero = TryInto::<u64>::try_into(unread_count)
      .unwrap()
      .try_into()
      .unwrap();
    fetch_and_handle_unread(id.clone(), as_non_zero).await?;
  };

  // Unlikely to happen unless their API is failing (server overload?), but still possible
  let log = "\
(Notice) Failed to mark convo as read. \
Command will be added again, but likely will be ignored by HashSet. \
There exists, however, a small chance that it will be handled twice. \
This possibly will cause the user to receive a duplicate response.";

  drop(
    read_convo::act(id)
      .await
      .map_err(|_| event!(Level::WARN, log)),
  );

  Ok(())
}

async fn fetch_and_handle_unread(
  convo_id: String,
  unread_count: NonZeroU64,
) -> Result<(), Error<anyhow::Error>> {
  let unread_messages: Vec<_> = fetch_messages::act(convo_id.clone(), unread_count)
    .await
    .map_err(|e| anyhow!(e))?
    .into_iter()
    .filter_map(handle_union)
    .collect();
  for message in unread_messages {
    match message {
      OutputMessagesItem::ChatBskyConvoDefsMessageView(view) => {
        let Object { data, .. } = *view;
        add_pending(convo_id.clone(), data).await;
      }
      OutputMessagesItem::ChatBskyConvoDefsDeletedMessageView(_) => {
        event!(Level::DEBUG, "Message has been unsent. Ignoring.");
      }
    }
  }
  Ok(())
}
