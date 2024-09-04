use std::time::Duration;

use session::{Bsky, BSKY};
use tokio::{sync::RwLockWriteGuard, time::sleep};

pub mod get_last_post_time;
pub mod notify_watchers;

static MINIMUM_DELAY: u64 = 10; // 10 Milliseconds

pub async fn minimum_delay() {
  let mut context: RwLockWriteGuard<'_, Bsky> = BSKY.get().await.write().await;
  let current = chrono::offset::Utc::now().naive_utc();
  #[allow(clippy::unwrap_used)] // Guaranteed not to overflow
  let elapsed: u64 = current
    .signed_duration_since(context.last_action)
    .num_milliseconds()
    .try_into()
    .unwrap();
  if elapsed < MINIMUM_DELAY {
    sleep(Duration::from_millis(MINIMUM_DELAY - elapsed)).await;
  }
  context.last_action = current;
}
