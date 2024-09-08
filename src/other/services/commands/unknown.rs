//! # `Unknown` command.
//! 
//! Implements the `Command` trait. Does not implement the `Parseable` trait,
//! as it is not meant to be parsed, but rather to be used as a fallback for
//! unknown commands.
//! Returns a hard-coded message to the user indicating that the command is unknown.
// TODO: Possibly make a configuration file for the message?

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
