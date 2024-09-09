use super::Bsky;
use atrium_api::{
  app::bsky::actor::{defs::ProfileViewDetailedData, get_profile},
  types::{string::AtIdentifier, Object},
  xrpc,
};
use ipld_core::ipld::Ipld;
use thiserror::Error as ThisError;
use xrpc::error::Error as XrpcError;

use crate::BskyReq;

#[derive(ThisError, Debug)]
pub enum Error {}

/// Get the profile of a specific user.
///
/// # Errors
///
/// Will return any unhandled request errors.
pub async fn act(actor: AtIdentifier) -> Result<ProfileViewDetailedData, super::Error<Error>> {
  Request { actor }.act().await
}

struct Request {
  actor: AtIdentifier,
}
impl BskyReq for Request {
  type ReqParams = get_profile::Parameters;
  type ReqOutput = ProfileViewDetailedData;
  type ReqError = get_profile::Error;
  type HandledError = Error;

  fn get_params(self) -> Self::ReqParams {
    get_profile::Parameters {
      data: get_profile::ParametersData { actor: self.actor },
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
      .actor
      .get_profile(params)
      .await
  }

  fn handle_xrpc_custom_error(e: Self::ReqError) -> Option<super::Error<Error>> {
    match e {
      // Unreachable: This request has no custom errors
    }
  }
}
