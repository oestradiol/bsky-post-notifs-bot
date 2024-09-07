use std::{cmp, time::Duration};

use atrium_api::types::string::AtIdentifier;
use chrono::{DateTime, Utc};
use repositories::watched_user;

use tokio::time::sleep;
use tracing::{event, Level};

use bsky::get_last_post_time;
use utils::{handle_api_failure, Did};

use crate::{notify, user_unwatched};

pub async fn begin() {
  let watching = watched_user::get_watching().await;

  event!(Level::INFO, "Now watching all users' posts.");

  for watched_did in watching {
    tokio::spawn(new(watched_did));
  }
}

static WATCH_DELAY: i64 = 15; // 15 Seconds
#[expect(clippy::missing_panics_doc)] // False positive
pub async fn new(watched_did: Did) {
  let watched_did_as_at = watched_did.parse::<AtIdentifier>().unwrap();
  let mut failures_in_a_row = 0;
  let mut last_notified_watchers: DateTime<Utc> = Utc::now();
  loop {
    if !watched_user::is_watched(&watched_did).await {
      event!(
        Level::INFO,
        "User {watched_did} is no longer being watched."
      );
      break;
    }

    event!(Level::DEBUG, "Checking for new posts from {watched_did}...");

    let before_task = Utc::now();
    match get_last_post_time::act(watched_did_as_at.clone()).await {
      Err(bsky::Error::Api) => {
        event!(
          Level::WARN,
          "(Notice) Error fetching last post time for {watched_did}."
        );
        if handle_api_failure(&mut failures_in_a_row).await {
          tokio::spawn(user_unwatched::handle(watched_did.clone(), false));
          break;
        }
        continue;
      }
      Err(bsky::Error::BskyBug) => {
        event!(Level::ERROR, "Stopping watching {watched_did}...");
        tokio::spawn(user_unwatched::handle(watched_did.clone(), false));
        break;
      }
      Err(bsky::Error::Other(get_last_post_time::Error::UserOptedOut)) => {
        event!(
          Level::INFO,
          "{watched_did} has opted out of the watchlist. Will stop watching."
        );
        tokio::spawn(user_unwatched::handle(watched_did.clone(), true));
        break;
      }
      Err(bsky::Error::Other(get_last_post_time::Error::ZeroPosts)) => {
        event!(Level::DEBUG, "API returned zero posts for {watched_did}.");
      }
      Ok(output) => {
        event!(Level::DEBUG, "{watched_did}'s last post at: {:?}.", output);
        if output > last_notified_watchers {
          last_notified_watchers = output;
          tokio::spawn(notify::watcher::many(watched_did.clone(), None, true));
        }
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
