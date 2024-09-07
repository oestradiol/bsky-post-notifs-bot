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

pub async fn act() -> Result<(BskyAgent, Did), BskyError> {
  let path = (*WORKSPACE_DIR).join(format!("{}-config.json", *BOT_USERNAME));
  let file_store = FileStore::new(path);

  match try_load_from_config(&file_store).await {
    Ok(agent) => Ok(agent),
    Err(e) => handle_bsky_error(&file_store, e).await,
  }
}

async fn do_auth(file_store: &FileStore) -> Result<(BskyAgent, Did), BskyError> {
  let agent = BskyAgent::builder().build().await?;
  agent.login(*BOT_USERNAME, *BOT_PASSWORD).await?;
  let config = agent.to_config().await;
  #[expect(clippy::unwrap_used)] // Just logged in
  let did = config.session.as_ref().unwrap().did.clone();
  config.save(file_store).await?;
  Ok((agent, Arc::from(String::from(did))))
}

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
