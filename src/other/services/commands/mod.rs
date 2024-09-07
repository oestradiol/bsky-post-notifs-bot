mod help;
mod invalid;
mod list_watched;
mod unknown;
mod unwatch;
mod watch;

use std::{future::Future, pin::Pin};

use atrium_api::{
  app::bsky::richtext::facet::{Main, MainData, MainFeaturesItem, MentionData},
  types::{
    string::{AtIdentifier, Did},
    Object,
  },
};
use help::Help;
use invalid::Invalid;
use list_watched::ListWatched;
use std::fmt::Debug;
use tracing::{event, Level};
use unknown::Unknown;
use unwatch::Unwatch;
use utils::handle_union;
use watch::Watch;

pub(crate) type Result<T> = core::result::Result<T, bsky::Error<anyhow::Error>>;
pub(crate) type PinnedFut<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub trait Command: Debug {
  fn process(self: Box<Self>, sender_id: Did) -> PinnedFut<Result<String>>;
  fn box_dyn(self) -> Box<dyn Command + Send>
  where
    Self: Sized + Send + 'static,
  {
    Box::new(self) as Box<dyn Command + Send>
  }
}
pub(crate) trait Parseable: Command {
  async fn parse(facets: Option<Vec<Main>>) -> Result<Self>
  where
    Self: Sized + Command;
}

/// # Errors
///
/// Watch and Unwatch might fail to resolve handles and dids
pub async fn parse(
  text: &str,
  facets: Option<Vec<Object<MainData>>>,
) -> Result<Box<dyn Command + Send>> {
  let text = text.trim();
  if !text.starts_with('!') {
    event!(
      Level::DEBUG,
      "Message does not start with a command prefix: {text}"
    );
    return Ok(Invalid.box_dyn());
  }

  let mut parts = text.split_whitespace();
  #[allow(clippy::unwrap_used)] // Checked above
  let command = parts.next().unwrap().to_lowercase();

  let res = match command.as_str() {
    "!help" => Help.box_dyn(),
    "!watch" => Watch::parse(facets).await?.box_dyn(),
    "!unwatch" => Unwatch::parse(facets).await?.box_dyn(),
    "!list_watched" => ListWatched.box_dyn(),
    _ => Unknown.box_dyn(),
  };
  Ok(res)
}

fn extract_mentions(facets: Vec<Main>) -> Vec<AtIdentifier> {
  facets
    .into_iter()
    .flat_map(
      |Object {
         data: MainData { features, .. },
         ..
       }| features,
    )
    .filter_map(handle_union)
    .filter_map(|facet| match facet {
      MainFeaturesItem::Mention(mention) => {
        let Object {
          data: MentionData { did },
          ..
        } = *mention;
        Some(did)
      }
      _ => None,
    })
    .map(AtIdentifier::from)
    .collect()
}
