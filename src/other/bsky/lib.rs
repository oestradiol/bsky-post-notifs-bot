pub mod get_last_post_time;
pub mod get_messages;
pub mod get_profile;
pub mod get_profiles;
pub mod get_unread_convos;
pub mod get_user_convo;
mod login;
pub mod read_convo;
pub mod send_message;

use async_once::AsyncOnce;
use atrium_api::types::Object;
use atrium_xrpc::{
  error::{Error as XrpcError, XrpcError as XrpcErrorResponse, XrpcErrorKind},
  http::StatusCode,
};
use bsky_sdk::BskyAgent;
use lazy_static::lazy_static;
use std::{sync::Arc, time::Duration};
use thiserror::Error as ThisError;
use tokio::{sync::RwLock, time::sleep};
use tracing::{event, Level};
use utils::Did;

lazy_static! {
  pub static ref BSKY: AsyncOnce<RwLock<Bsky>> = AsyncOnce::new(Bsky::init());
}
static MAXIMUM_RETRIES: i8 = 4;
static RETRY_DELAY: u64 = 15;

pub struct Bsky {
  agent: Option<Arc<BskyAgent>>,
  agent_id: Option<Did>,
  // last_action: DateTime<Utc>,
}
impl Bsky {
  async fn init() -> RwLock<Self> {
    // #[expect(clippy::unwrap_used)] // Constant, should never fail
    // let last_action = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
    let (agent, did) = Self::retry_until_get_agent().await;
    let bsky = Self {
      agent: Some(Arc::new(agent)),
      agent_id: Some(did),
      // last_action,
    };
    RwLock::new(bsky)
  }

  #[expect(clippy::cognitive_complexity)]
  async fn retry_until_get_agent() -> (BskyAgent, Did) {
    event!(Level::INFO, "Logging in...");

    let agent;
    let mut retries = MAXIMUM_RETRIES;
    loop {
      match login::act().await {
        Ok(res) => {
          agent = res;
          break;
        }
        Err(e) => event!(Level::WARN, "(Notice) Failed to login: {:?}", e),
      }

      retries -= 1;
      assert!(
        retries >= 0,
        "Failed to login after {MAXIMUM_RETRIES} retries. Exiting..."
      );
      event!(
        Level::INFO,
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
    bsky.agent_id = None;
  }

  async fn revalidate_agent() {
    let mut bsky = BSKY.get().await.write().await;
    let (agent, did) = Self::retry_until_get_agent().await;
    bsky.agent = Some(Arc::new(agent));
    bsky.agent_id = Some(did);
  }

  #[expect(clippy::missing_panics_doc)]
  pub async fn get_agent() -> Arc<BskyAgent> {
    let bsky = BSKY.get().await.read().await;
    match &bsky.agent {
      Some(agent) => agent.clone(),
      None => {
        drop(bsky);
        Self::revalidate_agent().await;
        #[expect(clippy::unwrap_used)] // Defined immediately above
        BSKY
          .get()
          .await
          .read()
          .await
          .agent
          .as_ref()
          .unwrap()
          .clone()
      }
    }
  }

  #[expect(clippy::missing_panics_doc)]
  pub async fn get_agent_did() -> Did {
    let bsky = BSKY.get().await.read().await;
    match &bsky.agent_id {
      Some(did) => did.clone(),
      None => {
        if bsky.agent.is_some() {
          event!(
            Level::WARN,
            "FIXME: Agent ID not found, but agent is present!"
          );
        }
        drop(bsky);
        Self::revalidate_agent().await;
        #[expect(clippy::unwrap_used)] // Defined immediately above
        BSKY
          .get()
          .await
          .read()
          .await
          .agent_id
          .as_ref()
          .unwrap()
          .clone()
      }
    }
  }
}

#[derive(ThisError, Debug)]
pub enum Error<Other> {
  #[error("API error")]
  Api,
  #[error("Invalid Bluesky data")]
  BskyBug,
  #[error(transparent)]
  Other(#[from] Other),
}

trait BskyReq {
  type ReqParams: Clone;
  type ReqOutput;
  type ReqError: std::fmt::Debug;
  type HandledError: std::error::Error + std::fmt::Debug;
  const PER_REQ_MAX_RETRIES: u8 = 3;
  const ON_FAILURE_DELAY: u64 = 150; // 150 Milliseconds

  fn get_params(self) -> Self::ReqParams;
  async fn request(
    params: Self::ReqParams,
  ) -> Result<Object<Self::ReqOutput>, atrium_xrpc::Error<Self::ReqError>>;
  fn handle_xrpc_custom_error(err: Self::ReqError) -> Option<Error<Self::HandledError>>;

  async fn act(self) -> Result<Self::ReqOutput, Error<Self::HandledError>>
  where
    Self: Sized,
  {
    // Commented out until I get rate limited at least once
    // Unlikely to ever happen given the ping
    // minimum_delay().await;

    let mut failed_attempts = 0;

    let params = Self::get_params(self);
    loop {
      match Self::attempt(params.clone()).await {
        Err(Some(Error::Api) | None) => {
          if !Self::handle_error(&mut failed_attempts).await {
            return Err(Error::Api);
          }
          continue;
        }
        Err(Some(Error::BskyBug)) => {
          if !Self::handle_error(&mut failed_attempts).await {
            return Err(Error::BskyBug);
          }
          continue;
        }
        Err(Some(err)) => return Err(err),
        Ok(output) => return Ok(output),
      }
    }
  }

  // Returns true if the error was handled, false if it was not
  async fn handle_error(failed_attempts: &mut u8) -> bool {
    if *failed_attempts < Self::PER_REQ_MAX_RETRIES {
      *failed_attempts += 1;
      sleep(Duration::from_millis(Self::ON_FAILURE_DELAY)).await;
    } else {
      // Shouldn't ever really reach this, after a 401, the agent should be revalidated successfully
      // or else the program will have stopped. But we leave this here just in case ig?
      return false;
    }
    true
  }

  async fn attempt(
    params: Self::ReqParams,
  ) -> Result<Self::ReqOutput, Option<Error<Self::HandledError>>> {
    let err = match Self::request(params).await {
      Ok(output) => return Ok(output.data),
      Err(XrpcError::XrpcResponse(XrpcErrorResponse::<Self::ReqError> { status, error })) => {
        let status = status.as_u16();
        if status == StatusCode::UNAUTHORIZED.as_u16() {
          Bsky::invalidate_agent().await;
          None
        } else if let Some(XrpcErrorKind::Custom(e)) = error {
          Self::handle_xrpc_custom_error(e)
        } else {
          event!(
            Level::WARN,
            "(Notice) Failed to issue request, API Error. Status Code: {status}. Error: {error:?}."
          );
          Some(Error::Api)
        }
      }
      Err(XrpcError::HttpRequest(e)) => {
        event!(
          Level::WARN,
          "(Notice) Failed to issue request, API Error: {e}"
        );
        Some(Error::Api)
      }
      Err(XrpcError::HttpClient(e)) => {
        event!(
          Level::WARN,
          "(Notice) Failed to issue request, API Error: {e}"
        );
        Some(Error::Api)
      }
      Err(XrpcError::SerdeJson(e)) => {
        event!(
          Level::WARN,
          "(Notice) Failed to issue request, Bsky Error: {e}"
        );
        Some(Error::BskyBug)
      }
      Err(XrpcError::SerdeHtmlForm(e)) => {
        event!(
          Level::WARN,
          "(Notice) Failed to issue request, Bsky Error: {e}"
        );
        Some(Error::BskyBug)
      }
      Err(XrpcError::UnexpectedResponseType) => {
        event!(
          Level::WARN,
          "(Notice) Failed to issue request, Bsky Error: Unexpected Response Type"
        );
        Some(Error::BskyBug)
      }
    };
    Err(err)
  }
}

// static MINIMUM_DELAY: u64 = 10; // 10 Milliseconds

// async fn minimum_delay() {
//   let mut context: RwLockWriteGuard<'_, Bsky> = BSKY.get().await.write().await;
//   let current = chrono::offset::Utc::now();
//   #[expect(clippy::unwrap_used)] // Guaranteed not to overflow
//   let elapsed: u64 = current
//     .signed_duration_since(context.last_action)
//     .num_milliseconds()
//     .try_into()
//     .unwrap();
//   if elapsed < MINIMUM_DELAY {
//     sleep(Duration::from_millis(MINIMUM_DELAY - elapsed)).await;
//   }
//   context.last_action = current;
// }
