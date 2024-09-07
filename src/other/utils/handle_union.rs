use atrium_api::types::Union;
use tracing::{event, Level};

pub fn handle_union<Refs>(union: Union<Refs>) -> Option<Refs> {
  match union {
    Union::Refs(refs) => Some(refs),
    Union::Unknown(unknown) => {
      event!(Level::WARN, "(Notice) Unknown union data: {:?}", unknown);
      None
    }
  }
}
