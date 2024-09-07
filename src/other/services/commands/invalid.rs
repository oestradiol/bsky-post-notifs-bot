use atrium_api::types::string::Did;

use super::{Command, PinnedFut, Result};

#[derive(Debug)]
pub struct Invalid;
impl Command for Invalid {
  fn process(self: Box<Self>, _: Did) -> PinnedFut<Result<String>> {
    Box::pin(async move {
      Ok(
        "\
If you're trying to issue a command, please use the command prefix:
- `!<command>`

You can get a list of available commands with:
- `!help`\
        "
        .to_string(),
      )
    })
  }
}
