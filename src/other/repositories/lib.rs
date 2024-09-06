pub mod watched_user;

use async_once::AsyncOnce;
use lazy_static::lazy_static;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Error, Sqlite, SqlitePool, Transaction};
use std::time::Duration;
use tracing::{event, Level};

use environment::{DATABASE_URL, DB_CONN_POOL_MAX};
use tokio::time;

/// # Loadable<T>
/// Represents the type of a T that can be loaded from the
/// database, where the query can either fail, resolve to None
/// or resolve to Some(T).
///
/// ## Variants
/// - Err(e)           query failed
/// - Ok(None)         T not found
/// - Ok(Some(T))      T found
pub type Loadable<T> = sqlx::Result<Option<T>>;

/// # Transaction
/// Represents an Sqlite transaction
pub type AppTransaction = Transaction<'static, Sqlite>;

lazy_static! {
  static ref DB_CONTEXT: AsyncOnce<Database> = AsyncOnce::new(Database::init());
}

pub struct Database {
  pool: SqlitePool,
}
impl Database {
  /// # Panics
  ///
  /// Panics when connection pool fails to initialize.
  #[allow(clippy::cognitive_complexity)]
  async fn init() -> Self {
    if Sqlite::database_exists(*DATABASE_URL).await.unwrap_or(false) {
      event!(Level::INFO, "Database found: {}", *DATABASE_URL);
    } else {
      event!(Level::INFO, "Creating database: {}", *DATABASE_URL);
      match Sqlite::create_database(*DATABASE_URL).await {
        Ok(()) => event!(Level::DEBUG, "Success!"),
        Err(e) => panic!("Failed to create db! Error: {e}"),
      }
    }
    
    let pool = SqlitePoolOptions::new()
      .max_connections(*DB_CONN_POOL_MAX)
      .connect(*DATABASE_URL)
      .await
      .unwrap_or_else(|e| panic!("Failed to connect to Sqlite DB! Error: {e}"));

    Self { pool }
  }

  pub async fn get_pool() -> &'static SqlitePool {
    &DB_CONTEXT.get().await.pool
  }

  /// # Errors
  ///
  /// Fails when a transaction cannot be started.
  pub async fn get_tx() -> Result<AppTransaction, Error> {
    DB_CONTEXT.get().await.pool.begin().await
  }

  #[allow(clippy::redundant_pub_crate)] // Select macro propagates this
  pub async fn disconnect() {
    let db_countdown = time::sleep(Duration::from_secs(15));
    let db_shutdown = async {
      event!(Level::INFO, "Closing database connections (max. 15s)...");
      DB_CONTEXT.get().await.pool.close().await;
      event!(Level::INFO, "Database connections closed!");
    };

    tokio::select! {
      () = db_countdown => {},
      () = db_shutdown => {},
    }
  }
}
