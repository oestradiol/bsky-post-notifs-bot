use std::{cmp, time::Duration};

use tokio::time::sleep;
use tracing::{event, Level};

static INCREMENTS_LIMIT: u64 = 60; // 60 Seconds
static MINUTES_LIMIT: u64 = 30; // 30 Minutes
static MAX_FAILURES: u64 = {
  let seconds_spent_incrementing = INCREMENTS_LIMIT * (INCREMENTS_LIMIT + 1) / 2;
  let minutes_as_secs = MINUTES_LIMIT * 60;
  if minutes_as_secs < seconds_spent_incrementing {
    INCREMENTS_LIMIT
  } else {
    INCREMENTS_LIMIT + (minutes_as_secs - seconds_spent_incrementing) / INCREMENTS_LIMIT
  }
};

/// Handles API failures by sleeping for an incrementing amount of time.
/// The bot will retry in incrementing intervals of 1s, up to `INCREMENTS_LIMIT`,
/// for a maximum of `MINUTES_LIMIT`.
///
/// # Returns
/// A bool indicating whether the maximum retries have been reached.
pub async fn handle_api_failure(failures_in_a_row: &mut u64) -> bool {
  if *failures_in_a_row >= MAX_FAILURES {
    event!(Level::ERROR, "Maximum retries reached! Aborting...");
    return true;
  }

  *failures_in_a_row += 1;
  sleep(Duration::from_secs(cmp::min(
    *failures_in_a_row,
    INCREMENTS_LIMIT,
  )))
  .await;

  false
}
