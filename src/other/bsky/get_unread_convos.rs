use super::Bsky;
use atrium_api::{
  agent::bluesky::AtprotoServiceType,
  chat::bsky::convo::{defs::ConvoView, list_convos},
  types::Object,
  xrpc,
};
use ipld_core::ipld::Ipld;
use thiserror::Error as ThisError;
use xrpc::error::Error as XrpcError;

use crate::BskyReq;

#[derive(ThisError, Debug)]
pub enum Error {}

/// # Errors
///
/// Will return any unhandled request errors.
pub async fn act() -> Result<Vec<ConvoView>, super::Error<Error>> {
  let mut curr_cursor = None;
  let mut all_unread_convos = Vec::new();
  loop {
    // TODO: Maybe handle instead, maybe simply ignore error and try again since cursor here? But limit attempts.
    let list_convos::OutputData { cursor, convos } = Request { curr_cursor }.act().await?;
    let total_count = convos.len();

    let unread_convos: Vec<_> = convos.into_iter().filter(|c| c.unread_count > 0).collect();
    let unread_count = unread_convos.len();

    all_unread_convos.extend(unread_convos);

    if unread_count < total_count || cursor.is_none() {
      break;
    }
    curr_cursor = cursor;
  }

  Ok(all_unread_convos)
}

struct Request {
  curr_cursor: Option<String>,
}
impl BskyReq for Request {
  type ReqParams = list_convos::Parameters;
  type ReqOutput = list_convos::OutputData;
  type ReqError = list_convos::Error;
  type HandledError = Error;

  fn get_params(self) -> Self::ReqParams {
    list_convos::Parameters {
      data: list_convos::ParametersData {
        cursor: self.curr_cursor,
        #[allow(clippy::unwrap_used)] // Safe because it's a constant
        limit: Some(15.try_into().unwrap()),
      },
      extra_data: Ipld::Null,
    }
  }

  async fn request(
    params: Self::ReqParams,
  ) -> Result<Object<Self::ReqOutput>, XrpcError<Self::ReqError>> {
    Bsky::get_agent()
      .await
      .api_with_proxy(
        #[allow(clippy::unwrap_used)] // Hard coded
        "did:web:api.bsky.chat".parse().unwrap(),
        AtprotoServiceType::BskyChat,
      )
      .chat
      .bsky
      .convo
      .list_convos(params)
      .await
  }

  fn handle_xrpc_custom_error(e: Self::ReqError) -> Option<super::Error<Error>> {
    match e {
       // Unreachable: This request has no custom errors
    }
  }
}
