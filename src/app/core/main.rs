mod on_shutdown;

use environment::{TURN_OFF_WATCHED_NOTIFS, WORKSPACE_DIR};
use on_shutdown::with_graceful_shutdown;
use repositories::Database;
use services::jobs;
use tracing::{event, Level};

#[cfg(unix)]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Main entry point for the application.
///
/// This function initializes the logging system, runs the database migrations, and starts the
/// command listener and issuer. It also starts watching users for new posts.
#[tokio::main]
async fn main() {
  dotenv::from_filename(WORKSPACE_DIR.join(".env")).ok();
  // Logging - The variable is needed for the lifetime of the program
  let (discord_worker, _log_guards) = utils::init_logging().await;

  // Database auto migration
  event!(Level::INFO, "Running DB migrations...");
  sqlx::migrate!()
    .run(Database::get_pool().await)
    .await
    .unwrap_or_else(|e| panic!("Failed to migrate DB! Error: {e}"));

  event!(Level::INFO, "Application starting!");

  if *TURN_OFF_WATCHED_NOTIFS {
    event!(Level::INFO, "Bot will not notify users that they are being watched. Feature disabled in environment.");
  } else {
    event!(Level::INFO, "Bot is set to notify users that they are being watched.");
  }

  #[expect(clippy::redundant_pub_crate)] // Select macro propagates this
  let commands_fut = async {
    let listener = jobs::command_listener::begin();
    let issuer = jobs::command_issuer::begin();

    tokio::select! {
      () = listener => {},
      () = issuer => {},
    }

    event!(Level::WARN, "Commands are no longer being processed...");
  };
  tokio::spawn(commands_fut);
  tokio::spawn(jobs::user_watcher::begin());

  with_graceful_shutdown(discord_worker).await;
}
