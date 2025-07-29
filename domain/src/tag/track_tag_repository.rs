use anyhow::Result;
use sqlx::PgTransaction;

/// 曲から全てのタグを削除
pub async fn delete_all_tags_from_track<'c>(
    tx: &mut PgTransaction<'c>,
    track_id: i32,
) -> Result<()> {
    sqlx::query!("DELETE FROM track_tags WHERE track_id = $1", track_id,)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
