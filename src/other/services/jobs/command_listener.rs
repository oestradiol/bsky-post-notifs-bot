use std::{cmp, time::Duration};

use bsky::get_unread_convos;
use chrono::Utc;
use tokio::time::sleep;
use tracing::{event, Level};
use utils::handle_api_failure;

use crate::unanswered_convos;

pub static WATCH_DELAY: i64 = 2; // 2 Seconds

/// Method for listening for new commands from users.
/// Will fetch the last unread convos from time to time (`WATCH_DELAY`),
/// and then handle each message accordingly.
/// Has a basic compensation mechanism that tries to, on average and as much as possible,
/// wait for exactly `WATCH_DELAY` seconds between each loop.
/// Also has a mechanism to handle persistent API failures, cancelling the job if the
/// error appears to be unrecoverable, logging the error.
/// 
/// Note: This job failing means the `command_issuer` job will also stop, given that
/// they're both being used in a tokio::select! block.
/// Also important to note that if the user sends multiple messages in a row, they will
/// be handled in order, but the bot will not wait for the previous commands to be issued,
/// and will ignore any previous messages if the user sends a new one. This is to prevent
/// the bot from being stuck on a single user's messages (DDoS).
#[expect(clippy::cognitive_complexity)]
#[expect(clippy::missing_panics_doc)] // False positive because of unwrap
pub async fn begin() {
  event!(Level::INFO, "Now listening to user commands.");

  let mut failures_in_a_row = 0;
  loop {
    let before_task = Utc::now();
    match get_unread_convos::act().await {
      Err(bsky::Error::Api) => {
        event!(Level::WARN, "(Notice) Error fetching last dms.");
        if handle_api_failure(&mut failures_in_a_row).await {
          break;
        }
        continue;
      }
      Err(bsky::Error::BskyBug) => {
        event!(Level::ERROR, "Stopping listening to dms...");
        break;
      }
      Err(bsky::Error::Other(e)) => {
        match e {
           // Unreachable: This request has no custom errors
        }
      }
      Ok(dms) => {
        // Awaits to handle this batch first, or else concurrency
        // issues might happen where the same convo is handled twice
        unanswered_convos::handle_many(dms.into_iter().map(|c| c.data).collect()).await;
      }
    }
    failures_in_a_row = 0;
    let after_task = Utc::now();

    let task_delta = after_task
      .signed_duration_since(before_task)
      .num_milliseconds();

    #[expect(clippy::unwrap_used)] // cmp::max checked so unwrap is safe
    let time_left = cmp::max(WATCH_DELAY * 1000 - task_delta, 0)
      .try_into()
      .unwrap();
    sleep(Duration::from_millis(time_left)).await;
  }
}
