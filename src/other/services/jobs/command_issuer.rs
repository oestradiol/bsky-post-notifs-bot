use std::time::Duration;

use crate::{
  jobs::command_listener,
  pending_messages::{self, PENDING_MESSAGES},
};

use tokio::time::sleep;
use tracing::{event, Level};

/// Method for handling all the new commands that were previously cached by
/// the bot (check `command_listener`).
/// Will fetch the last unread convos from time to time (`WATCH_DELAY`),
/// and then handle each message accordingly.
/// Command failure will be logged, but the bot will not notify the user about it,
/// after all, if the command failed it's because it wasn't able to notify the user
/// to begin with.
/// 
/// Note: This job will only ever stop if `command_listener` job stop, given that
/// they're both being used in a tokio::select! block.
#[expect(clippy::missing_panics_doc)] // False positive because of unwrap
pub async fn begin() {
  event!(Level::INFO, "Now handling pending messages.");

  loop {
    PENDING_MESSAGES
      .write()
      .await
      .drain()
      .map(|(k, v)| {
        tokio::spawn(async {
          pending_messages::process(k, v)
            .await
            .map_err(|e| event!(Level::WARN, "(Notice) Failed to handle pending message. User will have to reissue command. Error: {e}"))
        })
      })
      .for_each(drop);

    #[expect(clippy::unwrap_used)] // Constant
    sleep(Duration::from_secs(
      command_listener::WATCH_DELAY.try_into().unwrap(),
    ))
    .await;
  }
}
