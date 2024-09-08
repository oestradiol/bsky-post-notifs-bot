pub mod watched_user;

use async_once::AsyncOnce;
use lazy_static::lazy_static;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Error, Sqlite, SqlitePool, Transaction};
use std::time::Duration;
use tracing::{event, Level};

use environment::{owned_var_or, owned_var_or_else};
use tokio::time;

/// Represents the type of a T that can be loaded from the
/// database, where the query can either fail, resolve to None
/// or resolve to Some(T).
///
/// # Variants
/// - Err(e)           query failed
/// - Ok(None)         T not found
/// - Ok(Some(T))      T found
pub(crate) type Loadable<T> = sqlx::Result<Option<T>>;

/// # Transaction
/// Represents an Sqlite transaction
pub(crate) type AppTransaction = Transaction<'static, Sqlite>;

lazy_static! {
  /// Current DB connection pool
  static ref DB: AsyncOnce<Database> = AsyncOnce::new(Database::init());
}

/// The database connection pool.
pub struct Database {
  pool: SqlitePool,
}
impl Database {
  /// Initiallizes by checking if the database file exists, creating it if it
  /// doesn't, and then connecting to it, initializing the connection pool.
  /// 
  /// # Panics
  ///
  /// Panics when connection pool fails to initialize.
  #[expect(clippy::cognitive_complexity)]
  async fn init() -> Self {
    let db_url = owned_var_or_else("DATABASE_URL", || String::from("sqlite://data.db"));

    if Sqlite::database_exists(&db_url)
      .await
      .unwrap_or(false)
    {
      event!(Level::INFO, "Database found: {}", &db_url);
    } else {
      event!(Level::INFO, "Creating database: {}", &db_url);
      match Sqlite::create_database(&db_url).await {
        Ok(()) => event!(Level::DEBUG, "Successfully created new DB file!"),
        Err(e) => panic!("Failed to create db! Error: {e}"),
      }
    }

    let conn_pool_max: u32 = owned_var_or("DB_CONN_POOL_MAX", 100);

    let pool = SqlitePoolOptions::new()
      .max_connections(conn_pool_max)
      .connect(&db_url)
      .await
      .unwrap_or_else(|e| panic!("Failed to connect to Sqlite DB! Error: {e}"));

    Self { pool }
  }

  /// Method to get a reference to the database connection pool.
  pub async fn get_pool() -> &'static SqlitePool {
    &DB.get().await.pool
  }

  /// Method to get a transaction from the database connection pool.
  /// 
  /// # Errors
  ///
  /// Fails when a transaction cannot be started.
  pub(crate) async fn get_tx() -> Result<AppTransaction, Error> {
    DB.get().await.pool.begin().await
  }

  /// Method for gracefully disconnecting from the database.
  #[expect(clippy::redundant_pub_crate)] // Select macro propagates this
  pub async fn disconnect() {
    let db_countdown = time::sleep(Duration::from_secs(15));
    let db_shutdown = async {
      event!(Level::INFO, "Closing database connections (max. 15s)...");
      DB.get().await.pool.close().await;
      event!(Level::INFO, "Database connections closed!");
    };

    tokio::select! {
      () = db_countdown => {},
      () = db_shutdown => {},
    }
  }
}
