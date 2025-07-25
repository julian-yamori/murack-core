use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::tag::DbTrackTagRepository;
use sqlx::PgTransaction;

/// DbTrackTagRepositoryの本実装
#[derive(new)]
pub struct DbTrackTagRepositoryImpl {}

#[async_trait]
impl DbTrackTagRepository for DbTrackTagRepositoryImpl {
    /// 曲から全てのタグを削除
    async fn delete_all_tags_from_track<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track_id: i32,
    ) -> Result<()> {
        sqlx::query!("DELETE FROM track_tags WHERE track_id = $1", track_id,)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}
