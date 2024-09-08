//! # `Help` command.
//! 
//! Implements the `Command` trait. Does not implement the `Parseable` trait,
//! as there is no relevant information in the command that is used here.
//! 
//! Returns a hard-coded message to the user indicating all signatures for each command.

use atrium_api::types::string::Did;

use super::{Command, PinnedFut, Result};

#[derive(Debug)]
pub struct Help;
impl Command for Help {
  fn process(self: Box<Self>, _: Did) -> PinnedFut<Result<String>> {
    Box::pin(async move {
      Ok(
        "\
Available commands:
- `!watch @user_1 @user_2 (...)`
- `!unwatch @user_1 @user_2 (...)`
- `!list_watched`
- `!help`\
        ".to_string(),
      )
    })
  }
}
