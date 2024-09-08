use std::sync::Arc;

use bsky_sdk::{
  agent::config::{Config, FileStore},
  api::xrpc::http::StatusCode,
  error::GenericXrpcError,
  BskyAgent, Error as BskyError,
};
use environment::{BOT_PASSWORD, BOT_USERNAME, WORKSPACE_DIR};
use tracing::{event, Level};
use utils::Did;

/// Attempt to login to the Bsky API.
/// First, it will try to load the agent from a config file.
/// Then, if that fails, it will attempt to login and create a new session.
/// 
/// # Returns
/// A tuple of the Bsky agent and the DID of the logged in user.
/// 
/// # Errors
/// If the login fails.
pub async fn act() -> Result<(BskyAgent, Did), BskyError> {
  let path = (*WORKSPACE_DIR).join(format!("{}-config.json", *BOT_USERNAME));
  let file_store = FileStore::new(path);

  match try_load_from_config(&file_store).await {
    Ok(agent) => Ok(agent),
    Err(e) => handle_bsky_error(&file_store, e).await,
  }
}

/// Attempt to login to the Bsky API and saves the session to a config file.
async fn do_auth(file_store: &FileStore) -> Result<(BskyAgent, Did), BskyError> {
  let agent = BskyAgent::builder().build().await?;
  agent.login(*BOT_USERNAME, *BOT_PASSWORD).await?;
  let config = agent.to_config().await;
  #[expect(clippy::unwrap_used)] // Just logged in
  let did = config.session.as_ref().unwrap().did.clone();
  config.save(file_store).await?;
  Ok((agent, Arc::from(String::from(did))))
}

/// Attempt to load the Bsky agent from a config file, if it exists.
async fn try_load_from_config(file_store: &FileStore) -> Result<(BskyAgent, Did), BskyError> {
  if let Ok(config) = Config::load(file_store).await {
    #[expect(clippy::unwrap_used)] // Restored from session
    let did = config.session.as_ref().unwrap().did.clone();
    let agent = BskyAgent::builder().config(config).build().await?;
    event!(
      Level::DEBUG,
      "Recovered previous configs for user {}!",
      *BOT_USERNAME
    );
    Ok((agent, Arc::from(String::from(did))))
  } else {
    do_auth(file_store).await
  }
}

/// Handle any errors from the Bsky API.
/// In case it returns that the token is invalid, it will attempt to re-authenticate.
async fn handle_bsky_error(
  file_store: &FileStore,
  e: BskyError,
) -> Result<(BskyAgent, Did), BskyError> {
  match e {
    BskyError::Xrpc(e) => {
      if let GenericXrpcError::Response { status, .. } = *e {
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
          return do_auth(file_store).await;
        }
      }

      Err(BskyError::Xrpc(e))
    }
    e => Err(e),
  }
}
