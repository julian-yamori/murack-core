use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::db::DbTransaction;

/// song_tagsテーブルのDAO
#[async_trait]
pub trait SongTagsDao {
    /// レコードを新規登録
    async fn insert<'c>(&self, tx: &mut DbTransaction<'c>, song_id: i32, tag_id: i32)
    -> Result<()>;

    /// 曲IDでレコードを削除
    async fn delete_by_song_id<'c>(&self, tx: &mut DbTransaction<'c>, song_id: i32) -> Result<()>;
}

/// SongTagsDaoの本実装
pub struct SongTagsDaoImpl {}

#[async_trait]
impl SongTagsDao for SongTagsDaoImpl {
    /// レコードを新規登録
    async fn insert<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        tag_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO track_tags (track_id, tag_id) VALUES ($1, $2)",
            song_id,
            tag_id,
        )
        .execute(&mut **tx.get())
        .await?;

        Ok(())
    }

    /// 曲IDでレコードを削除
    async fn delete_by_song_id<'c>(&self, tx: &mut DbTransaction<'c>, song_id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM track_tags WHERE track_id = $1", song_id,)
            .execute(&mut **tx.get())
            .await?;

        Ok(())
    }
}
