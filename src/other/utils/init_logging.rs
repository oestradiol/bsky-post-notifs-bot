use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use environment::{LOG_DIRECTORY, STDOUT_LOG_SEVERITY};

/// # Panics
///
/// When logging fails to initialize.
pub async fn init_logging() -> (WorkerGuard, WorkerGuard) {
  color_eyre::install().unwrap_or_default();

  let canonical = super::canonicalize_unexistent(*LOG_DIRECTORY)
    .unwrap_or_else(|| panic!("Failed to canonicalize path!"));
  tokio::fs::create_dir_all(&canonical)
    .await
    .unwrap_or_else(|e| panic!("Failed to create canonical directory: {e}. Path: {canonical:?}"));

  #[allow(clippy::unwrap_used)]
  let filter = EnvFilter::builder()
    .with_default_directive((*STDOUT_LOG_SEVERITY).into())
    .from_env()
    .unwrap_or_else(|e| panic!("Invalid directives for tracing subscriber: {e}."))
    .add_directive("hyper_util::client=info".parse().unwrap()) // Hyper client is too verbose
    .add_directive("reqwest::connect=info".parse().unwrap()); // Reqwest client is too verbose

  let file_appender = tracing_appender::rolling::daily(canonical, "PostNotifs.log");
  let (non_blocking_file, guard0) = tracing_appender::non_blocking(file_appender);
  let (non_blocking_stdout, guard1) = tracing_appender::non_blocking(std::io::stdout());

  let file_log = tracing_subscriber::fmt::layer().with_writer(non_blocking_file);
  let stdout_log = tracing_subscriber::fmt::layer()
    .pretty()
    .with_writer(non_blocking_stdout);

  let layered = stdout_log.and_then(file_log).with_filter(filter);

  tracing_subscriber::registry().with(layered).init();

  (guard0, guard1)
}
