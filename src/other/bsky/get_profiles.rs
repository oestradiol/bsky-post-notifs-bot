use super::Bsky;
use atrium_api::{
  app::bsky::actor::{defs::ProfileViewDetailedData, get_profiles},
  types::{string::AtIdentifier, Object},
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
  actors: Vec<AtIdentifier>,
) -> Result<Vec<ProfileViewDetailedData>, super::Error<Error>> {
  Ok(
    Request { actors }
      .act()
      .await?
      .profiles
      .into_iter()
      .map(|o| o.data)
      .collect(),
  )
}

struct Request {
  actors: Vec<AtIdentifier>,
}
impl BskyReq for Request {
  type ReqParams = get_profiles::Parameters;
  type ReqOutput = get_profiles::OutputData;
  type ReqError = get_profiles::Error;
  type HandledError = Error;

  fn get_params(self) -> Self::ReqParams {
    get_profiles::Parameters {
      data: get_profiles::ParametersData {
        actors: self.actors,
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
      .actor
      .get_profiles(params)
      .await
  }

  fn handle_xrpc_custom_error(e: Self::ReqError) -> Option<super::Error<Error>> {
    match e {
      // Unreachable: This request has no custom errors
    }
  }
}
