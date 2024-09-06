use std::sync::Arc;

use crate::{AppTransaction, Loadable};

/// # Errors
/// 
/// Returns an error if the query fails.
pub async fn delete(tx: &mut AppTransaction, did: Arc<str>) -> Loadable<()> {
  let did = &*did;
  let rows = sqlx::query!(r#"DELETE FROM "WatchedUser" WHERE did = $1"#, did)
    .execute(&mut **tx)
    .await?
    .rows_affected();

  Ok(if rows > 0 { Some(()) } else { None })
}
