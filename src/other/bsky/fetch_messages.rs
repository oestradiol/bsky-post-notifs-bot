use std::num::NonZeroU64;

use super::Bsky;
use atrium_api::{
  agent::bluesky::AtprotoServiceType,
  chat::bsky::convo::get_messages,
  types::{LimitedNonZeroU8, Object, Union},
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
#[allow(clippy::missing_panics_doc)]
pub async fn act(
  convo_id: String,
  count: NonZeroU64,
) -> Result<Vec<Union<get_messages::OutputMessagesItem>>, super::Error<Error>> {
  #[allow(clippy::unwrap_used)] // Hard coded
  let mut batches: Vec<LimitedNonZeroU8<100>> =
    vec![100.try_into().unwrap(); (count.get() / 100) as usize];
  let remainder = (count.get() % 100) as u8;
  if remainder > 0 {
    #[allow(clippy::unwrap_used)] // NonZeroU64, should never fail since remainder [1, 100]
    batches.push(remainder.try_into().unwrap());
  }

  let mut curr_cursor = None;
  let mut all_unread_messages = Vec::new();
  for limit in batches {
    let get_messages::OutputData { cursor, messages } = Request {
      curr_cursor,
      convo_id: convo_id.clone(),
      limit,
    }
    .act()
    .await?; // TODO: Handle instead, maybe simply ignore error and try again since cursor here? But limit attempts.
    all_unread_messages.extend(messages);
    curr_cursor = cursor;
  }

  Ok(all_unread_messages)
}

struct Request {
  curr_cursor: Option<String>,
  convo_id: String,
  limit: LimitedNonZeroU8<100u8>,
}
impl BskyReq for Request {
  type ReqParams = get_messages::Parameters;
  type ReqOutput = get_messages::OutputData;
  type ReqError = get_messages::Error;
  type HandledError = Error;

  fn get_params(self) -> Self::ReqParams {
    get_messages::Parameters {
      data: get_messages::ParametersData {
        cursor: self.curr_cursor,
        limit: Some(self.limit),
        convo_id: self.convo_id,
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
      .get_messages(params)
      .await
  }

  fn handle_xrpc_custom_error(_: Self::ReqError) -> Option<super::Error<Error>> {
    unreachable!() // This request has no custom errors
  }
}
