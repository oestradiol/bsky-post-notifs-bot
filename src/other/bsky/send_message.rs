use super::Bsky;
use atrium_api::{
  agent::bluesky::AtprotoServiceType,
  chat::bsky::convo::{
    defs::{MessageInput, MessageInputData, MessageViewData},
    send_message,
  },
  types::Object,
  xrpc,
};
use bsky_sdk::{rich_text::RichText, Error as BskyError};
use ipld_core::ipld::Ipld;
use thiserror::Error as ThisError;
use tracing::{event, Level};
use xrpc::error::Error as XrpcError;

use crate::BskyReq;

#[derive(ThisError, Debug)]
pub enum Error {
  #[error("ATrium bug")]
  AtriumBug,
}

/// # Errors
///
/// Will return any unhandled request errors.
pub async fn act(
  convo_id: String,
  message: String,
) -> Result<MessageViewData, super::Error<Error>> {
  let RichText { facets, text } = RichText::new_with_detect_facets(message)
    .await
    .map_err(handle_rich_text_error)?;
  let message = MessageInputData {
    facets,
    text,
    embed: None,
  };
  Request { convo_id, message }.act().await
}

struct Request {
  convo_id: String,
  message: MessageInputData,
}
impl BskyReq for Request {
  type ReqParams = send_message::Input;
  type ReqOutput = MessageViewData;
  type ReqError = send_message::Error;
  type HandledError = Error;

  fn get_params(self) -> Self::ReqParams {
    send_message::Input {
      data: send_message::InputData {
        convo_id: self.convo_id,
        message: MessageInput {
          data: self.message,
          extra_data: Ipld::Null,
        },
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
      .send_message(params)
      .await
  }

  fn handle_xrpc_custom_error(_: Self::ReqError) -> Option<super::Error<Error>> {
    unreachable!() // This request has no custom errors
  }
}

#[allow(clippy::cognitive_complexity)]
fn handle_rich_text_error(e: BskyError) -> super::Error<Error> {
  match e {
    BskyError::Xrpc(e) => {
      event!(Level::ERROR, "Error in RichText ATProto API Request: {e:?}");
      super::Error::Api
    }
    BskyError::ConfigLoad(e) | BskyError::ConfigSave(e) => {
      event!(
        Level::ERROR,
        "Got Config error for supposed request with no config: {e:?}"
      );
      super::Error::Other(Error::AtriumBug)
    }
    BskyError::Moderation(e) => {
      event!(
        Level::ERROR,
        "Got Moderation error for RichText Atrium API: {e:?}"
      );
      super::Error::Other(Error::AtriumBug)
    }
    BskyError::ApiType(e) => {
      event!(Level::ERROR, "Got API error for RichText Atrium API: {e:?}");
      super::Error::Other(Error::AtriumBug)
    }
    BskyError::NotLoggedIn => {
      event!(
        Level::ERROR,
        "Got NotLoggedIn for supposed unauthenticated request"
      );
      super::Error::Other(Error::AtriumBug)
    }
    BskyError::InvalidAtUri => {
      event!(Level::ERROR, "Got InvalidAtUri for RichText Atrium API");
      super::Error::Other(Error::AtriumBug)
    }
  }
}
