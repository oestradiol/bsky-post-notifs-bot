use std::path::PathBuf;

use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use environment::{owned_var_or, owned_var_or_else};

/// This method initializes the logging system for the application.
/// It reads the following environment variables:
/// - `LOG_DIRECTORY` - The directory where the logs will be stored. Defaults to `/var/log/post_watcher`.
/// - `LOG_SEVERITY` - The minimum severity level for logs. Defaults to `INFO`.
/// 
/// The logs are written to the console and to a file in the specified directory.
/// 
/// # Returns
/// Two WorkerGuards that need to live the entire lifetime of the application. 
/// 
/// # Panics
///
/// When logging fails to initialize.
pub async fn init_logging() -> (WorkerGuard, WorkerGuard) {
  color_eyre::install().unwrap_or_default();

  let log_directory =
    owned_var_or_else("LOG_DIRECTORY", || PathBuf::from("/var/log/post_watcher"));

  let canonical = super::canonicalize_unexistent(&log_directory)
    .unwrap_or_else(|| panic!("Failed to canonicalize path!"));
  tokio::fs::create_dir_all(&canonical)
    .await
    .unwrap_or_else(|e| panic!("Failed to create canonical directory: {e}. Path: {canonical:?}"));

  let log_severity = owned_var_or("LOG_SEVERITY", LevelFilter::INFO);

  #[expect(clippy::unwrap_used)] // Safe because it's a constant
  let filter = EnvFilter::builder()
    .with_default_directive(log_severity.into())
    .from_env()
    .unwrap_or_else(|e| panic!("Invalid directives for tracing subscriber: {e}."))
    .add_directive("hyper_util::client=info".parse().unwrap()) // Hyper client is too verbose
    .add_directive("reqwest::connect=info".parse().unwrap()); // Reqwest client is too verbose

  let file_appender = tracing_appender::rolling::daily(canonical, "PostNotifs.log");
  let (non_blocking_file, guard0) = tracing_appender::non_blocking(file_appender);
  let (non_blocking_stdout, guard1) = tracing_appender::non_blocking(std::io::stdout());

  let file_log = tracing_subscriber::fmt::layer()
    .compact()
    .with_writer(non_blocking_file);
  let stdout_log = tracing_subscriber::fmt::layer()
    .pretty()
    .with_writer(non_blocking_stdout);

  let layered = stdout_log.and_then(file_log).with_filter(filter);

  tracing_subscriber::registry().with(layered).init();

  (guard0, guard1)
}
