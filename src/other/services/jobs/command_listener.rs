use std::{cmp, time::Duration};

use bsky::get_unread_convos;
use chrono::Utc;
use tokio::time::sleep;
use tracing::{event, Level};
use utils::handle_api_failure;

use crate::unanswered_convos;

pub static WATCH_DELAY: i64 = 2; // 2 Seconds

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::missing_panics_doc)]
pub async fn begin() {
  event!(Level::INFO, "Now listening to user commands.");

  let mut failures_in_a_row = 0;
  loop {
    event!(Level::DEBUG, "Checking for new dms...");

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

    #[allow(clippy::unwrap_used)] // cmp::max checked so unwrap is safe
    let time_left = cmp::max(WATCH_DELAY * 1000 - task_delta, 0)
      .try_into()
      .unwrap();
    sleep(Duration::from_millis(time_left)).await;
  }
}
