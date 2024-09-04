use std::{sync::Arc, time::Duration};

use super::notify_watchers;
use chrono::{DateTime, Utc};
use repositories::watched_user::WATCHED_USERS;

use super::handle_user_unwatched;
use tokio::{task::JoinSet, time::sleep};
use tracing::{event, Level};

use bsky::get_last_post_time;
use utils::handle_api_failure;

pub async fn act() {
  let watched_lock = WATCHED_USERS.get().await.read().await;
  let watched: Vec<_> = watched_lock
    .iter()
    .map(|kv| (kv.0.clone(), kv.1.last_notified))
    .collect();
  drop(watched_lock);

  event!(Level::INFO, "Now peeking all users' notifications.");

  let mut set = JoinSet::new();
  for (user, last_peeked) in watched {
    set.spawn(peek_user(user, last_peeked));
  }
  while let Some(res) = set.join_next().await {
    let _ = res.map_err(|e| event!(Level::ERROR, "Failed to Join peek_user: {e:?}"));
  }
}

static WATCH_DELAY: u64 = 15; // 15 Seconds
pub async fn peek_user(user: Arc<str>, mut last_notified: DateTime<Utc>) {
  let mut failures_in_a_row = 0;

  loop {
    match get_last_post_time::act(user.clone()).await {
      Err(bsky::Error::Api) => {
        event!(Level::WARN, "Error fetching last post time for {user}.");
        if handle_api_failure(&mut failures_in_a_row).await {
          handle_user_unwatched::act(&user, false);
          break;
        }
        continue;
      }
      Err(bsky::Error::BskyBug) => {
        event!(Level::ERROR, "Stopping watching {user}...");
        break;
      }
      Err(bsky::Error::Other(get_last_post_time::Error::UserOptedOut)) => {
        event!(
          Level::INFO,
          "{user} has opted out of the watchlist. Will stop watching."
        );
        handle_user_unwatched::act(&user, true);
        break;
      }
      Err(bsky::Error::Other(get_last_post_time::Error::ZeroPosts)) => {
        event!(Level::DEBUG, "{user} currently has zero posts.");
      }
      Ok(output) => {
        event!(Level::DEBUG, "{user}'s last post at: {:?}.", output);
        if output > last_notified {
          last_notified = output;
          notify_watchers::r#try(user.clone()).await;
        }
      }
    }

    failures_in_a_row = 0;
    sleep(Duration::from_secs(WATCH_DELAY)).await;
  }
}
