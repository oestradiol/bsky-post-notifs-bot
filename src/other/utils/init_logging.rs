use std::path::PathBuf;

use debug_print::debug_println;
use regex::Regex;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_layer_discord::{DiscordConfig, DiscordLayer, EventFilters};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Layer};

pub use tracing_layer_discord::BackgroundWorker; // Re-export for the main function

use environment::{owned_var_or_else, owned_var_try};

/// This method initializes the logging system for the application.
/// It reads the following environment variables:
/// - `LOG_DIRECTORY` - The directory where the logs will be stored. Defaults to `/var/log/post_watcher`.
/// - `LOG_SEVERITY` - The minimum severity level for logs. Defaults to `INFO`.
///
/// The logs are written to the console and to a file in the specified directory.
///
/// # Returns
/// Two `WorkerGuards` that need to live the entire lifetime of the application.
///
/// # Panics
///
/// When logging fails to initialize.
pub async fn init_logging() -> (Option<BackgroundWorker>, (WorkerGuard, WorkerGuard)) {
  debug_println!("\nInitializing logging...\n");

  // Initializing color_eyre for better error handling
  color_eyre::install().unwrap_or_default();

  // Ensuring the log directory exists
  let log_dir = log_directory().await; 

  // Reading the log severity level
  let log_severity: String = owned_var_try("LOG_SEVERITY").unwrap();

  // Filtering verbose crates
  let filtered = vec!["h2",  "sqlx", "hyper", "hyper_util", "reqwest"];
  let env_filter = filter(&filtered, &log_severity);

  // Setting up the file and stdout appenders
  let file_appender = tracing_appender::rolling::daily(log_dir, "PostNotifsWatcher.log");
  let (non_blocking_file, guard0) = tracing_appender::non_blocking(file_appender);
  let (non_blocking_stdout, guard1) = tracing_appender::non_blocking(std::io::stdout());

  let file_log = tracing_subscriber::fmt::layer()
    .compact()
    .with_writer(non_blocking_file);
  let stdout_log = tracing_subscriber::fmt::layer()
    .pretty()
    .with_writer(non_blocking_stdout);

  let stdout_and_file_layer = stdout_log.and_then(file_log).with_filter(env_filter);

  let discord_worker = if let Ok(discord_webhook) = owned_var_try("DISCORD_WEBHOOK") {
    // If the webhook is provided, we add and an async background task for sending our Discord messages
    debug_println!("Discord webhook provided.\n");

    // Filtering out the verbose crates from this layer with its provided interface
    let filtered = filtered.into_iter().fold(String::new(), |acc, f| acc + f + "|");
    let event_filters = EventFilters::new(Some(vec![Regex::new(".*").unwrap()]), Some(vec![Regex::new(filtered.trim_end_matches('|')).unwrap()]));
    let (discord_layer, background_worker) = DiscordLayer::builder(
      "post-notifs-watcher".to_string(),
      event_filters,
    )
    .discord_config(DiscordConfig::new(discord_webhook))
    .level_filters(log_severity)
    .build();

    // Setting up the subscriber with the Discord layer and the stdout/file layer
    let subscriber = tracing_subscriber::registry().with(discord_layer.and_then(stdout_and_file_layer));
    tracing::subscriber::set_global_default(subscriber).unwrap();

    Some(background_worker)
  } else {
    // If the webhook is not provided, we only use the stdout/file layer
    debug_println!("Discord webhook not found.\n");

    // 
    let subscriber = tracing_subscriber::registry().with(stdout_and_file_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    None
  };

  debug_println!("Success!\n");
  debug_println!("-------------------------------------------------------\n");
  (discord_worker, (guard0, guard1))
}

// This function creates the log directory and returns its path.
async fn log_directory() -> PathBuf {
  let log_dir = owned_var_or_else("LOG_DIRECTORY", || PathBuf::from("/var/log/post_watcher"));

  let canonical = super::canonicalize_unexistent(&log_dir)
    .unwrap_or_else(|| panic!("Failed to canonicalize path!"));

  tokio::fs::create_dir_all(&canonical)
    .await
    .unwrap_or_else(|e| panic!("Failed to create canonical directory: {e}. Path: {canonical:?}"));

  canonical
}

// This function creates the filter for the logging system.
fn filter(filter_entries: &[&str], log_severity: &str) -> EnvFilter {
  debug_println!("Defining EnvFilter...\n");
  #[expect(clippy::unwrap_used)] // Safe because it's a constant
  let filter = EnvFilter::builder()
    .with_default_directive(log_severity.parse::<LevelFilter>().unwrap().into())
    .from_env()
    .unwrap_or_else(|e| panic!("Invalid directives for tracing subscriber: {e}."));

  filter_entries.iter().fold(filter, |acc, s|  {
    acc.add_directive(format!("{s}=warn").parse().unwrap())
  })
}