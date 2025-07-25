use anyhow::Result;
use murack_core_domain::{db::DbTransaction, path::LibSongPath};

/// 指定されたpathのレコードが存在するか確認
pub async fn exists_path<'c>(tx: &mut DbTransaction<'c>, path: &LibSongPath) -> Result<bool> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE path = $1"#,
        path.as_str(),
    )
    .fetch_one(&mut **tx.get())
    .await?;

    Ok(count > 0)
}
