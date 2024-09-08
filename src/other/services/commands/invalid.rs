//! # `Invalid` command.
//! 
//! Implements the `Command` trait. Does not implement the `Parseable` trait,
//! as it is not meant to be parsed, but rather to be used as a fallback for
//! invalid commands.
//! Returns a hard-coded message to the user indicating that the command is invalid.
// TODO: Possibly make a configuration file for the message?

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
