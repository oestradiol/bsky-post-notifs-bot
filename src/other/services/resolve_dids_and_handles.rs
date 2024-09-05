use std::collections::HashSet;

use atrium_api::{
  app::bsky::actor::defs::ProfileViewDetailedData,
  types::string::{AtIdentifier, Did, Handle},
};
use bsky::get_profiles;

pub async fn act(
  actors: Vec<AtIdentifier>,
) -> Result<(HashSet<Did>, HashSet<Handle>), bsky::Error<get_profiles::Error>> {
  Ok(get_profiles::act(actors).await?.into_iter().fold(
    (HashSet::new(), HashSet::new()),
    |mut acc, ProfileViewDetailedData { did, handle, .. }| {
      acc.0.insert(did);
      acc.1.insert(handle);
      acc
    },
  ))
}
