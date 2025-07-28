use anyhow::Result;
use sqlx::PgTransaction;

use crate::path::LibraryTrackPath;

/// 指定されたpathのレコードが存在するか確認
pub async fn exists_path<'c>(tx: &mut PgTransaction<'c>, path: &LibraryTrackPath) -> Result<bool> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE path = $1"#,
        path.as_ref() as &str,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(count > 0)
}
