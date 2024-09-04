mod peek_users;

use environment::WORKSPACE_DIR;
use tracing::{event, Level};

#[cfg(unix)]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() {
  dotenv::from_filename(WORKSPACE_DIR.join(".env")).ok();
  // Logging - The variables are needed for the lifetime of the program
  let _log_guards = utils::init_logging().await;

  event!(Level::INFO, "Application starting!");

  peek_users::act().await;
}
