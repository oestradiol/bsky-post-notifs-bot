use std::collections::HashSet;

use atrium_api::{
  app::bsky::actor::defs::ProfileViewDetailedData,
  types::string::{AtIdentifier, Handle},
};
use bsky::get_profiles;

/// Auxiliary method to resolve a list of actors into their handles.
pub async fn act(
  actors: Vec<AtIdentifier>,
) -> Result<HashSet<Handle>, bsky::Error<get_profiles::Error>> {
  let (dids, handles) = actors
    .into_iter()
    .fold((HashSet::new(), HashSet::new()), |mut acc, a| match a {
      AtIdentifier::Did(_) => {
        acc.0.insert(a);
        acc
      }
      AtIdentifier::Handle(handle) => {
        acc.1.insert(handle);
        acc
      }
    });

  Ok(
    get_profiles::act(dids.into_iter().collect())
      .await?
      .into_iter()
      .fold(
        handles,
        |mut acc, ProfileViewDetailedData { handle, .. }| {
          acc.insert(handle);
          acc
        },
      ),
  )
}
