use super::Bsky;
use atrium_api::{
  agent::bluesky::AtprotoServiceType, chat::bsky::convo::update_read, types::Object, xrpc,
};
use ipld_core::ipld::Ipld;
use thiserror::Error as ThisError;
use xrpc::error::Error as XrpcError;

use crate::BskyReq;

#[derive(ThisError, Debug)]
pub enum Error {}

/// Method for updating the read status of a conversation.
/// 
/// # Errors
///
/// Will return any unhandled request errors.
pub async fn act(convo_id: String) -> Result<update_read::OutputData, super::Error<Error>> {
  Request { convo_id }.act().await
}

struct Request {
  convo_id: String,
}
impl BskyReq for Request {
  type ReqParams = update_read::Input;
  type ReqOutput = update_read::OutputData;
  type ReqError = update_read::Error;
  type HandledError = Error;

  fn get_params(self) -> Self::ReqParams {
    Self::ReqParams {
      data: update_read::InputData {
        convo_id: self.convo_id,
        message_id: None,
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
        #[expect(clippy::unwrap_used)] // Safe because it's a constant
        "did:web:api.bsky.chat".parse().unwrap(),
        AtprotoServiceType::BskyChat,
      )
      .chat
      .bsky
      .convo
      .update_read(params)
      .await
  }

  fn handle_xrpc_custom_error(e: Self::ReqError) -> Option<super::Error<Error>> {
    match e {
       // Unreachable: This request has no custom errors
    }
  }
}
