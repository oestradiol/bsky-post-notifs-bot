use repositories::Database;
use tokio::signal;
use tracing::{event, Level};

async fn before_shutdown() {
  event!(
    Level::INFO,
    "The bot is shutting down! Running shutdown routines..."
  );
  Database::disconnect().await;
}

pub async fn with_graceful_shutdown() {
  shutdown_signal().await;
  before_shutdown().await;
}

/// # Panics
///
/// Will panic if fails to install any of the signal handlers.
async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .unwrap_or_else(|e| panic!("Failed to install Ctrl+C handler! {e}"));
  };

  #[cfg(unix)]
  let term_or_int = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .unwrap_or_else(|e| panic!("Failed to install SIGTERM handler! {e}"))
      .recv()
      .await;
  };

  #[cfg(windows)]
  let term_or_int = async {
    signal::windows::ctrl_close()
      .unwrap_or_else(|e| panic!("Failed to install Windows SIGINT handler! {e}"))
      .recv()
      .await;
  };

  tokio::select! {
    () = ctrl_c => {},
    () = term_or_int => {},
  }
}
