use atrium_api::types::Union;
use tracing::{event, Level};

pub fn handle_union<Refs>(message: Union<Refs>) -> Option<Refs> {
  match message {
    Union::Refs(refs) => Some(refs),
    Union::Unknown(unknown) => {
      event!(Level::WARN, "(Notice) Unknown union data: {:?}", unknown);
      None
    }
  }
}
