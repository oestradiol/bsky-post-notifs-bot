use std::{cmp, sync::Arc, time::Duration};

use chrono::{DateTime, NaiveDateTime, Utc};
use repositories::watched_user::WATCHED_USERS;
use services::handle_user_unwatched;
use tokio::{task::JoinSet, time::sleep};
use tracing::{event, Level};

use bsky::{get_last_post_time, notify_watchers};

pub async fn act() {
  let watched_lock = WATCHED_USERS.get().await.read().await;
  let watched: Vec<_> = watched_lock
    .iter()
    .map(|kv| (kv.0.clone(), kv.1.last_peeked))
    .collect();
  drop(watched_lock);

  let mut set = JoinSet::new();
  for (user, last_peeked) in watched {
    set.spawn(peek_user(user, last_peeked));
  }
  while let Some(res) = set.join_next().await {
    let _ = res.map_err(|e| event!(Level::ERROR, "Failed to Join peek_user: {e:?}"));
  }
}

static WATCH_DELAY: u64 = 15; // 15 Seconds
                              // The bot will retry in incrementing intervals of 1s, up to INCREMENTS_LIMIT, for a maximum of MINUTES_LIMIT.
                              // Then, if the API failure is persistent, it'll give up on watching the user.
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

pub async fn peek_user(user: Arc<str>, mut last_peeked: NaiveDateTime) {
  let mut failures_in_a_row = 0;

  #[allow(clippy::unwrap_used)] // Constant, should never fail
  let mut last_post_time = DateTime::<Utc>::from_timestamp(0, 0).unwrap().naive_utc();
  loop {
    let result = get_last_post_time::act(user.clone()).await;
    match result {
      Err(get_last_post_time::Error::Api) => {
        event!(Level::WARN, "Error fetching last post time for {user}.");
        if failures_in_a_row >= MAX_FAILURES {
          event!(Level::ERROR, "Maximum retries reached! Aborting...");
          handle_user_unwatched::act(&user, false);
          break;
        }

        failures_in_a_row += 1;
        sleep(Duration::from_secs(cmp::min(
          failures_in_a_row,
          INCREMENTS_LIMIT,
        )))
        .await;
        continue;
      }
      Err(get_last_post_time::Error::BskyBug) => {
        event!(
          Level::ERROR,
          "A bug in Bluesky was encountered! Stopping this watch..."
        );
        break;
      }
      Err(get_last_post_time::Error::UserOptedOut) => {
        event!(
          Level::INFO,
          "{user} has opted out of the watchlist. Will stop watching."
        );
        handle_user_unwatched::act(&user, true);
        break;
      }
      Err(get_last_post_time::Error::ZeroPosts) => {
        event!(Level::INFO, "{user} currently has zero posts.");
      }
      Ok(output) => {
        event!(Level::INFO, "{user}'s last post at: {:?}.", output);
        last_post_time = output;
      }
    }

    if last_post_time > last_peeked {
      last_peeked = last_post_time;
      notify_watchers::r#try(user.clone()).await;
    }

    failures_in_a_row = 0;
    sleep(Duration::from_secs(WATCH_DELAY)).await;
  }
}
