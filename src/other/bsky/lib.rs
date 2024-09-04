use atrium_api::types::Object;
use atrium_xrpc::{
  error::{Error as XrpcError, XrpcError as XrpcErrorResponse, XrpcErrorKind},
  http::StatusCode,
};
use session::{Bsky, BSKY};
use std::time::Duration;
use thiserror::Error as ThisError;
use tokio::{sync::RwLockWriteGuard, time::sleep};
use tracing::{event, Level};

pub mod get_last_post_time;
pub mod get_unread_dms;
pub mod read_convo;

#[derive(ThisError, Debug)]
pub enum Error<Other> {
  #[error("API error")]
  Api,
  #[error("Invalid Bluesky data")]
  BskyBug,
  #[error(transparent)]
  Other(#[from] Other),
}

static PER_REQ_AUTH_MAX_RETRIES: u8 = 3;

trait BskyReq {
  type ReqParams: Clone;
  type ReqOutput;
  type ReqError: std::fmt::Debug;
  type HandledError: std::error::Error + std::fmt::Debug;

  fn get_params(self) -> Self::ReqParams;
  async fn request(
    params: Self::ReqParams,
  ) -> Result<Object<Self::ReqOutput>, atrium_xrpc::Error<Self::ReqError>>;
  fn handle_xrpc_custom_error(err: Self::ReqError) -> Option<Error<Self::HandledError>>;

  async fn act(self) -> Result<Self::ReqOutput, Error<Self::HandledError>>
  where
    Self: Sized,
  {
    minimum_delay().await;

    let mut failed_attempts = 0;

    let params = Self::get_params(self);
    loop {
      match Self::attempt(params.clone()).await {
        Err(None) => {
          event!(Level::DEBUG, "Failed to issue request, auth error");

          if failed_attempts < PER_REQ_AUTH_MAX_RETRIES {
            failed_attempts += 1;
          } else {
            return Err(Error::Api);
          }

          continue;
        }
        Ok(output) => return Ok(output),
        Err(Some(err)) => return Err(err),
      }
    }
  }

  async fn attempt(
    params: Self::ReqParams,
  ) -> Result<Self::ReqOutput, Option<Error<Self::HandledError>>> {
    let err = match Self::request(params).await {
      Ok(output) => return Ok(output.data),
      Err(XrpcError::XrpcResponse(XrpcErrorResponse::<Self::ReqError> { status, error })) => {
        let status = status.as_u16();
        if status == StatusCode::UNAUTHORIZED.as_u16() || status == StatusCode::FORBIDDEN.as_u16() {
          Bsky::invalidate_agent().await;
          None
        } else if let Some(XrpcErrorKind::Custom(e)) = error {
          Self::handle_xrpc_custom_error(e)
        } else {
          event!(
            Level::WARN,
            "Failed to issue request, API Error. Status Code: {status}. Error: {error:?}."
          );
          Some(Error::Api)
        }
      }
      Err(XrpcError::HttpRequest(e)) => {
        event!(Level::WARN, "Failed to issue request, API Error: {e}");
        Some(Error::Api)
      }
      Err(XrpcError::HttpClient(e)) => {
        event!(Level::WARN, "Failed to issue request, API Error: {e}");
        Some(Error::Api)
      }
      Err(XrpcError::SerdeJson(e)) => {
        event!(Level::WARN, "Failed to issue request, Bsky Error: {e}");
        Some(Error::BskyBug)
      }
      Err(XrpcError::SerdeHtmlForm(e)) => {
        event!(Level::WARN, "Failed to issue request, Bsky Error: {e}");
        Some(Error::BskyBug)
      }
      Err(XrpcError::UnexpectedResponseType) => {
        event!(
          Level::WARN,
          "Failed to issue request, Bsky Error: Unexpected Response Type"
        );
        Some(Error::BskyBug)
      }
    };
    Err(err)
  }
}

static MINIMUM_DELAY: u64 = 10; // 10 Milliseconds

async fn minimum_delay() {
  let mut context: RwLockWriteGuard<'_, Bsky> = BSKY.get().await.write().await;
  let current = chrono::offset::Utc::now();
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
