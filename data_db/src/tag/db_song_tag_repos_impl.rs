use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{db::DbTransaction, tag::DbSongTagRepository};

/// DbSongTagRepositoryの本実装
#[derive(new)]
pub struct DbSongTagRepositoryImpl {}

#[async_trait]
impl DbSongTagRepository for DbSongTagRepositoryImpl {
    /// 曲から全てのタグを削除
    async fn delete_all_tags_from_song<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()> {
        sqlx::query!("DELETE FROM track_tags WHERE track_id = $1", song_id,)
            .execute(&mut **tx.get())
            .await?;
        Ok(())
    }
}
