mod on_shutdown;

use environment::WORKSPACE_DIR;
use on_shutdown::with_graceful_shutdown;
use repositories::Database;
use tracing::{event, Level};

#[cfg(unix)]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() {
  dotenv::from_filename(WORKSPACE_DIR.join(".env")).ok();
  // Logging - The variables are needed for the lifetime of the program
  let _log_guards = utils::init_logging().await;

  // Database auto migration
  event!(Level::INFO, "Running DB migrations...");
  sqlx::migrate!()
    .run(Database::get_pool().await)
    .await
    .unwrap_or_else(|e| panic!("Failed to migrate DB! Error: {e}"));

  event!(Level::INFO, "Application starting!");

  #[allow(clippy::redundant_pub_crate)] // Select macro propagates this
  let commands_fut = async {
    let listen = services::listen_for_commands::act();
    let handle = services::handle_pending_messages::act();

    tokio::select! {
      () = listen => {},
      () = handle => {},
    }

    event!(Level::WARN, "Commands are no longer being processed...");
  };

  tokio::spawn(commands_fut);
  tokio::spawn(services::watch_users_for_posts::act());

  with_graceful_shutdown().await;
}
