use std::{collections::{BTreeMap, HashSet}, sync::Arc};

use chrono::Utc;
use types::entities::watched_user::{WatchedUser, Watcher};

use crate::{AppTransaction, Database};

use super::Data;

pub async fn get_all() -> sqlx::Result<BTreeMap<Arc<str>, Data>> {
  let mut tx = Database::get_tx().await?;
  let vec = get_all_from_db(&mut tx).await?;
  tx.commit().await?;

  let last_notified_watchers = Utc::now();

  let vec = vec
    .into_iter()
    .map(|u| {
      (
        u.did,
        Data {
          last_notified_watchers,
          watchers: u.watchers,
        },
      )
    })
    .collect();

  Ok(vec)
}

async fn get_all_from_db(tx: &mut AppTransaction) -> sqlx::Result<Vec<WatchedUser>> {
  let watched_user_record = sqlx::query!(r#"SELECT * FROM "WatchedUser""#)
    .fetch_all(&mut **tx)
    .await?;

  let mut watched_users = Vec::with_capacity(watched_user_record.capacity());
  for watched_user in watched_user_record {
    let did = Arc::from(watched_user.did);
    let watchers = serde_json::from_str::<HashSet<Watcher>>(&watched_user.watchers).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    watched_users.push(WatchedUser {
      did,
      watchers,
    });
  };

  Ok(watched_users)
}
