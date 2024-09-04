use std::{sync::Arc, time::Duration};

use atrium_api::{
  types::{string::AtIdentifier, Unknown::Object},
  xrpc::{self, http::StatusCode},
};
use bsky_sdk::api::app::bsky::feed::get_author_feed;
use chrono::NaiveDateTime;
use get_author_feed::Error as FeedError;
use ipld_core::ipld::Ipld;
use session::Bsky;
use thiserror::Error as ThisError;
use tokio::time::sleep;
use tracing::{event, Level};
use xrpc::error::{Error as XrpcError, XrpcError as XrpcErrorResponse, XrpcErrorKind};

use super::minimum_delay;

#[derive(ThisError, Debug)]
pub enum Error {
  #[error("API error")]
  Api,
  #[error("User had zero posts")]
  ZeroPosts,
  #[error("User has opted out of being watched")]
  UserOptedOut,
  #[error("Invalid Bluesky data")]
  BskyBug,
}

static TIME_MASK: &str = "%Y-%m-%dT%H:%M:%S%.fZ";

pub async fn act(user: Arc<str>) -> Result<NaiveDateTime, Error> {
  minimum_delay().await;

  let lastest_feed = get_feed(user).await?;
  let last_post = lastest_feed.feed.first().ok_or(Error::ZeroPosts)?;
  match &last_post.post.record {
    Object(map) => {
      let created_at = map.get("createdAt").map(|v| &**v);
      match created_at {
        Some(Ipld::String(time_str)) => chrono::NaiveDateTime::parse_from_str(time_str, TIME_MASK)
          .map_err(|_| {
            event!(
              Level::WARN,
              "Received invalid timestamp for createdAt: {:?}",
              time_str
            );
            Error::BskyBug
          }),
        unknown => {
          event!(
            Level::WARN,
            "Received unexpected value for createdAt: {:?}",
            unknown
          );
          Err(Error::BskyBug)
        }
      }
    }
    unknown => {
      event!(
        Level::WARN,
        "Received unexpected value for record: {:?}",
        unknown
      );
      Err(Error::BskyBug)
    }
  }
}

// It tries every 15s so it makes sense to give it a 15s window
static RETRY_DELAY: u64 = 5;
static MAXIMUM_FAILED_ATTEMPTS: u8 = 3;

async fn get_feed(user: Arc<str>) -> Result<get_author_feed::OutputData, Error> {
  let params = get_author_feed::Parameters {
    data: get_author_feed::ParametersData {
      #[allow(clippy::unwrap_used)] // It should be guaranteed that every user is a valid AtIdentifier, a DID, even.
      actor: user.parse::<AtIdentifier>().unwrap(),
      cursor: None,
      filter: None,
      #[allow(clippy::unwrap_used)] // Safe because it's a constant
      limit: Some(1.try_into().unwrap()),
    },
    extra_data: Ipld::Null,
  };

  let mut failed_attempts = 0;

  loop {
    match try_get_feed(params.clone()).await {
      Err(None) => {
        event!(Level::DEBUG, "Failed to get feed, auth error");

        if failed_attempts < MAXIMUM_FAILED_ATTEMPTS {
          failed_attempts += 1;
          event!(Level::DEBUG, "Retrying in {RETRY_DELAY}s...");
          sleep(Duration::from_secs(RETRY_DELAY)).await;
        } else {
          return Err(Error::Api);
        }

        continue;
      }
      Ok(feed) => return Ok(feed),
      Err(Some(err)) => return Err(err),
    }
  }
}

async fn try_get_feed(
  params: get_author_feed::Parameters,
) -> Result<get_author_feed::OutputData, Option<Error>> {
  let res = Bsky::get_agent()
    .await
    .api
    .app
    .bsky
    .feed
    .get_author_feed(params)
    .await;

  let err = match res {
    Ok(output) => return Ok(output.data),
    Err(XrpcError::XrpcResponse(XrpcErrorResponse::<FeedError> {
      error: Some(XrpcErrorKind::Custom(FeedError::BlockedByActor(_))),
      ..
    })) => Some(Error::UserOptedOut),
    Err(XrpcError::XrpcResponse(XrpcErrorResponse::<FeedError> { status, .. })) => {
      let status = status.as_u16();
      if status == StatusCode::UNAUTHORIZED.as_u16() || status == StatusCode::FORBIDDEN.as_u16() {
        Bsky::invalidate_agent().await;
        None
      } else {
        Some(Error::Api)
      }
    }
    Err(XrpcError::HttpRequest(_) | XrpcError::HttpClient(_)) => Some(Error::Api),
    Err(
      XrpcError::SerdeJson(_) | XrpcError::SerdeHtmlForm(_) | XrpcError::UnexpectedResponseType,
    ) => Some(Error::BskyBug),
  };
  Err(err)
}
