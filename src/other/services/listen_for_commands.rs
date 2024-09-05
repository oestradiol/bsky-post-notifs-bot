use std::time::Duration;

use super::handle_unanswered_convos;
use bsky::get_unread_convos;
use tokio::time::sleep;
use tracing::{event, Level};
use utils::handle_api_failure;

static WATCH_DELAY: u64 = 5; // 5 Seconds

#[allow(clippy::cognitive_complexity)]
pub async fn act() {
  event!(Level::INFO, "Now listening to user commands.");

  let mut failures_in_a_row = 0;

  loop {
    match get_unread_convos::act().await {
      Err(bsky::Error::Api) => {
        event!(Level::WARN, "Error fetching last dms.");
        if handle_api_failure(&mut failures_in_a_row).await {
          break;
        }
        continue;
      }
      Err(bsky::Error::BskyBug) => {
        event!(Level::ERROR, "Stopping listening to dms...");
        break;
      }
      Err(bsky::Error::Other(_)) => {
        unreachable!() // This request has no custom errors
      }
      Ok(dms) => {
        if !dms.is_empty() {
          handle_unanswered_convos::act(dms).await;
        }
      }
    }

    failures_in_a_row = 0;
    sleep(Duration::from_secs(WATCH_DELAY)).await;
  }
}
