mod login;

use std::{sync::Arc, time::Duration};

use async_once::AsyncOnce;
use bsky_sdk::BskyAgent;
use chrono::{DateTime, NaiveDateTime, Utc};
use lazy_static::lazy_static;
use tracing::{event, Level};

use tokio::{sync::RwLock, time::sleep};

lazy_static! {
  pub static ref BSKY: AsyncOnce<RwLock<Bsky>> = AsyncOnce::new(Bsky::init());
}
static MAXIMUM_RETRIES: i8 = 20;
static RETRY_DELAY: u64 = 15;

pub struct Bsky {
  agent: Option<Arc<BskyAgent>>,
  pub last_action: NaiveDateTime,
}
impl Bsky {
  async fn init() -> RwLock<Self> {
    #[allow(clippy::unwrap_used)] // Constant, should never fail
    let last_action = DateTime::<Utc>::from_timestamp(0, 0).unwrap().naive_utc();
    let bsky = Self {
      agent: Some(Arc::new(Self::retry_until_get_agent().await)),
      last_action,
    };
    RwLock::new(bsky)
  }

  #[allow(clippy::cognitive_complexity)]
  async fn retry_until_get_agent() -> BskyAgent {
    event!(Level::INFO, "Logging in...");

    let agent: BskyAgent;
    let mut retries = MAXIMUM_RETRIES;
    loop {
      match login::act().await {
        Ok(res) => {
          agent = res;
          break;
        }
        Err(e) => event!(Level::WARN, "Failed to login: {:?}", e),
      }

      retries -= 1;
      assert!(
        retries >= 0,
        "Failed to login after {MAXIMUM_RETRIES} retries. Exiting..."
      );
      event!(
        Level::WARN,
        "Trying again in {RETRY_DELAY} seconds. (Retries left: {})",
        retries
      );
      sleep(Duration::from_secs(RETRY_DELAY)).await;
    }
    agent
  }

  pub async fn invalidate_agent() {
    let mut bsky = BSKY.get().await.write().await;
    bsky.agent = None;
  }

  pub async fn get_agent() -> Arc<BskyAgent> {
    let bsky = BSKY.get().await.read().await;
    match &bsky.agent {
      Some(agent) => agent.clone(),
      None => {
        drop(bsky);
        let mut bsky = BSKY.get().await.write().await;
        bsky.agent = Some(Arc::new(Self::retry_until_get_agent().await));
        bsky.agent.as_ref().unwrap().clone()
      }
    }
  }
}
