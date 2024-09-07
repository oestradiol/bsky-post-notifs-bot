use super::Bsky;
use atrium_api::{
  types::{string::AtIdentifier, Object},
  xrpc::error::Error as XrpcError,
};
use bsky_sdk::api::app::bsky::feed::get_author_feed;
use chrono::{DateTime, Utc};
use ipld_core::ipld::Ipld;
use thiserror::Error as ThisError;
use tracing::{event, Level};

use crate::BskyReq;

#[derive(ThisError, Debug)]
pub enum Error {
  #[error("User had zero posts")]
  ZeroPosts,
  #[error("User has opted out of being watched")]
  UserOptedOut,
}

/// # Errors
///
/// Will return any unhandled request errors.
/// 
/// TODO: Fetch from now *until* last cursor (using the cursor would return posts before that)
///       And then implement `posts_with_replies` filter - returning "is_replies_only"
///       You can use that to notify only `watch_replies` users and know if should notify both.
pub async fn act(actor: AtIdentifier) -> Result<DateTime<Utc>, super::Error<Error>> {
  let time_str = Request { actor }
    .act()
    .await?
    .cursor
    .ok_or(super::Error::Other(Error::ZeroPosts))?;
  time_str
    .parse()
    .map_err(|_| {
      event!(
        Level::WARN,
        "(Notice) Received invalid timestamp for createdAt: {:?}",
        time_str
      );
      super::Error::BskyBug
    })
}

struct Request {
  actor: AtIdentifier,
}
impl BskyReq for Request {
  type ReqParams = get_author_feed::Parameters;
  type ReqOutput = get_author_feed::OutputData;
  type ReqError = get_author_feed::Error;
  type HandledError = Error;

  fn get_params(self) -> Self::ReqParams {
    Self::ReqParams {
      data: get_author_feed::ParametersData {
        actor: self.actor,
        cursor: None,
        filter: Some("posts_and_author_threads".to_string()), // TODO: Has to be changed to `posts_with_replies` on opt in
        #[expect(clippy::unwrap_used)] // Safe because it's a constant
        limit: Some(1.try_into().unwrap()),
      },
      extra_data: Ipld::Null,
    }
  }

  async fn request(
    params: Self::ReqParams,
  ) -> Result<Object<Self::ReqOutput>, XrpcError<Self::ReqError>> {
    Bsky::get_agent()
      .await
      .api
      .app
      .bsky
      .feed
      .get_author_feed(params)
      .await
  }

  fn handle_xrpc_custom_error(err: Self::ReqError) -> Option<super::Error<Error>> {
    match err {
      Self::ReqError::BlockedByActor(_) | Self::ReqError::BlockedActor(_) => {
        event!(Level::INFO, "User has opted out of being watched.");
        Some(super::Error::Other(Error::UserOptedOut))
      }
    }
  }
}
