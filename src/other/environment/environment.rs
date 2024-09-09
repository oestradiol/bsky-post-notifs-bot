//! In this file, we define the environment variables that are used multiple times
//! in the program, and therefore are better left as static variables.
//!
//! There are other environment variables that are only used once:
//! - `DISCORD_WEBHOOK` - The Discord webhook URL for logging.
//!   * Does not have a default value. Used at `init_logging`.
//! - `LOG_SEVERITY` - Severity level for the log file and stdout.
//!   * Defaults to `INFO`. Used at `utils::init_logging`.
//! - `LOG_DIRECTORY` - The directory where the log files are stored.
//!   * Defaults to `/var/log/post_watcher`. Used at `utils::init_logging`.
//! - `DATABASE_URL` - The URL to the database.
//!   * Defaults to `sqlite://data.db`. Used at `Database::init`.
//! - `DB_CONN_POOL_MAX` - The maximum number of connections to the database.
//!   * Defaults to `100`. Used at `Database::init`.

use std::path::Path;

#[cfg(debug_assertions)]
use anyhow::anyhow;
use lazy_static::lazy_static;

use crate::{owned_var_try, try_leak, var};

// Environment-agnostic variables

lazy_static! {
  /// Variable that determines whether the bot should send notifications to the user when
  /// they are watched and when they are unwatched. If the variable is set to anything,
  /// it will be considered as `false`. If it is unset, the feature is on by default.
  pub static ref TURN_OFF_WATCHED_NOTIFS: bool = owned_var_try::<String>("TURN_OFF_WATCHED_NOTIFS").is_ok();
  /// The bot username on The Atmosphere.
  pub static ref BOT_USERNAME: &'static str = var::<String, _>("BOT_USERNAME");
  /// The bot password or app password.
  pub static ref BOT_PASSWORD: &'static str = var::<String, _>("BOT_PASSWORD");
}

// Development environment variables

#[cfg(debug_assertions)]
lazy_static! {
  /// The cwd of the program on development.
  pub static ref WORKSPACE_DIR: &'static Path = (|| {
    let child_path_u8 = std::process::Command::new(env!("CARGO"))
      .arg("locate-project")
      .arg("--workspace")
      .arg("--message-format=plain")
      .output()?
      .stdout;
    let child_path_str = std::str::from_utf8(&child_path_u8)?.trim();
    let final_path = Path::new(child_path_str).parent()
      .ok_or_else(|| anyhow!("Couldn't find the parent directory of the workspace"))?;
    Ok::<&Path, anyhow::Error>(try_leak(final_path.to_path_buf())?)
  })().map_err(|e| panic!("Failed to set WORKSPACE_DIR: {e}")).unwrap();
}

// Production environment variables

#[cfg(not(debug_assertions))]
lazy_static! {
  /// The cwd of the program on production.
  pub static ref WORKSPACE_DIR: &'static Path = try_leak(Path::new(".")).unwrap();
}
