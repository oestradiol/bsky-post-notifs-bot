use std::collections::HashSet;

use atrium_api::{
  app::bsky::richtext::facet::Main,
  types::string::{Did, Handle},
};

use crate::{resolve_dids_and_handles, watch_new_users};

use super::{Command, Parseable, PinnedFut, Result};

#[derive(Debug)]
pub enum Watch {
  ParseSuccess(HashSet<Did>, HashSet<Handle>),
  ParseFail,
}
impl Parseable for Watch {
  async fn parse(facets: Option<Vec<Main>>) -> Result<Self> {
    let facets = match facets {
      None => return Ok(Self::ParseFail),
      Some(facets) => facets,
    };

    let at_ids = super::extract_mentions(facets);
    if at_ids.is_empty() {
      return Ok(Self::ParseFail);
    }

    let (dids, handles) = resolve_dids_and_handles::act(at_ids)
      .await
      .map_err(|e| anyhow::anyhow!(e))?;
    Ok(Self::ParseSuccess(dids, handles))
  }
}
impl Command for Watch {
  fn process(self: Box<Self>, sender_did: Did) -> PinnedFut<Result<String>> {
    Box::pin(async move {
      match *self {
        Self::ParseSuccess(dids, handles) => {
          watch_new_users::act(sender_did, dids).await;

          Ok(
            handles
              .into_iter()
              .fold("Now watching users:".to_string(), |acc, handle| {
                format!("{}\n- @{}", acc, handle.as_ref())
              }),
          )
        }
        Self::ParseFail => Ok("Please make sure to mention at least one user.".to_string()),
      }
    })
  }
}
