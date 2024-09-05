use std::{cmp, sync::Arc, time::Duration};

use super::notify_watchers;
use atrium_api::types::string::AtIdentifier;
use chrono::{DateTime, Utc};
use repositories::watched_user::WATCHED_USERS;

use super::handle_user_unwatched;
use tokio::time::sleep;
use tracing::{event, Level};

use bsky::get_last_post_time;
use utils::handle_api_failure;

pub async fn act() {
  let watched_lock = WATCHED_USERS.get().await.read().await;
  let watched: Vec<_> = watched_lock
    .iter()
    .map(|kv| (kv.0.clone(), kv.1.last_notified_watchers))
    .collect();
  drop(watched_lock);

  event!(Level::INFO, "Now watching all users' posts.");

  for (user_did, last_notified_watchers) in watched {
    tokio::spawn(watch_user(user_did, last_notified_watchers));
  }
}

static WATCH_DELAY: i64 = 15; // 15 Seconds
#[allow(clippy::missing_panics_doc)] // False positive
pub async fn watch_user(user_did: Arc<str>, mut last_notified_watchers: DateTime<Utc>) {
  let mut failures_in_a_row = 0;

  loop {
    event!(Level::DEBUG, "Checking for new posts from {user_did}...");

    let before_task = Utc::now();
    // TODO: Maybe create a wrapper to enforce this safety?
    #[allow(clippy::unwrap_used)] // It should be guaranteed that every user is a valid DID
    match get_last_post_time::act(user_did.parse::<AtIdentifier>().unwrap()).await {
      Err(bsky::Error::Api) => {
        event!(Level::WARN, "Error fetching last post time for {user_did}.");
        if handle_api_failure(&mut failures_in_a_row).await {
          tokio::spawn(handle_user_unwatched::act(user_did.clone(), false));
          break;
        }
        continue;
      }
      Err(bsky::Error::BskyBug) => {
        event!(Level::ERROR, "Stopping watching {user_did}...");
        tokio::spawn(handle_user_unwatched::act(user_did.clone(), false));
        break;
      }
      Err(bsky::Error::Other(get_last_post_time::Error::UserOptedOut)) => {
        event!(
          Level::INFO,
          "{user_did} has opted out of the watchlist. Will stop watching."
        );
        tokio::spawn(handle_user_unwatched::act(user_did.clone(), true));
        break;
      }
      Err(bsky::Error::Other(get_last_post_time::Error::ZeroPosts)) => {
        event!(Level::DEBUG, "{user_did} currently has zero posts.");
      }
      Ok(output) => {
        event!(Level::DEBUG, "{user_did}'s last post at: {:?}.", output);
        if output > last_notified_watchers {
          last_notified_watchers = output;
          tokio::spawn(notify_watchers::r#try(user_did.clone()));
        }
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
