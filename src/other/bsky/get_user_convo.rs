use super::Bsky;
use atrium_api::{
  agent::bluesky::AtprotoServiceType,
  chat::bsky::convo::get_convo_for_members,
  types::{string::Did, Object},
  xrpc,
};
use ipld_core::ipld::Ipld;
use thiserror::Error as ThisError;
use xrpc::error::Error as XrpcError;

use crate::BskyReq;

#[derive(ThisError, Debug)]
pub enum Error {}

/// Method for getting a conversation between the bot and a user.
///
/// # Errors
///
/// Will return any unhandled request errors.
#[expect(clippy::missing_panics_doc)] // False positive because of unwrap
pub async fn act(user_id: Did) -> Result<get_convo_for_members::OutputData, super::Error<Error>> {
  #[expect(clippy::unwrap_used)] // Safe, gotten from agent
  let our_did = Did::new(Bsky::get_agent_did().await.to_string()).unwrap();
  Request {
    members: vec![user_id, our_did],
  }
  .act()
  .await
}

struct Request {
  members: Vec<Did>,
}
impl BskyReq for Request {
  type ReqParams = get_convo_for_members::Parameters;
  type ReqOutput = get_convo_for_members::OutputData;
  type ReqError = get_convo_for_members::Error;
  type HandledError = Error;

  fn get_params(self) -> Self::ReqParams {
    Self::ReqParams {
      data: get_convo_for_members::ParametersData {
        members: self.members,
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
      .get_convo_for_members(params)
      .await
  }

  fn handle_xrpc_custom_error(e: Self::ReqError) -> Option<super::Error<Error>> {
    match e {
       // Unreachable: This request has no custom errors
    }
  }
}
