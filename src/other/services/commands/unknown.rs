use atrium_api::types::string::Did;

use super::{Command, PinnedFut, Result};

#[derive(Debug)]
pub struct Unknown;
impl Command for Unknown {
  fn process(self: Box<Self>, _: Did) -> PinnedFut<Result<String>> {
    Box::pin(async {
      Ok("Unknown command. You can get a list of available commands with `!help`.".to_string())
    })
  }
}
