use std::time::Duration;

use crate::{
  jobs::command_listener,
  pending_messages::{self, PENDING_MESSAGES},
};

use tokio::time::sleep;
use tracing::{event, Level};

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::missing_panics_doc)]
pub async fn begin() {
  event!(Level::INFO, "Now handling pending messages.");

  loop {
    event!(Level::DEBUG, "Checking for new pending messages...");

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

    #[allow(clippy::unwrap_used)] // Constant
    sleep(Duration::from_secs(
      command_listener::WATCH_DELAY.try_into().unwrap(),
    ))
    .await;
  }
}
