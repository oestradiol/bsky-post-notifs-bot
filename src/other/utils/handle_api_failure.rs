use std::{cmp, time::Duration};

use tokio::time::sleep;
use tracing::{event, Level};

use crate::constants::{INCREMENTS_LIMIT, MAX_FAILURES};

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
