use anyhow::Result;
use async_trait::async_trait;
use domain::db::DbTransaction;

/// song_artworkテーブルのDAO
#[async_trait]
pub trait SongArtworkDao {
    /// 曲IDを指定してアートワークIDを取得
    ///
    /// order順
    async fn select_artwork_id_by_song_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        track_id: i32,
    ) -> Result<Vec<i32>>;

    /// テーブル全体の件数を取得
    /// # todo
    /// テストでしか使ってない
    async fn count_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<u32>;

    /// 指定されたアートワークIDのレコード数を取得
    async fn count_by_artwork_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        artwork_id: i32,
    ) -> Result<u32>;

    /// 新規登録
    async fn insert<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        order: usize,
        artwork_id: i32,
        picture_type: u8,
        description: &str,
    ) -> Result<()>;

    /// 曲IDを指定して削除
    async fn delete_by_song_id<'c>(&self, tx: &mut DbTransaction<'c>, song_id: i32) -> Result<()>;
}

/// SongArtworkDaoの本実装
pub struct SongArtworkDaoImpl {}

#[async_trait]
impl SongArtworkDao for SongArtworkDaoImpl {
    /// 曲IDを指定してアートワークIDを取得
    ///
    /// order順
    async fn select_artwork_id_by_song_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        track_id: i32,
    ) -> Result<Vec<i32>> {
        let ids = sqlx::query_scalar!(
            "SELECT artwork_id FROM track_artworks WHERE track_id = $1 ORDER BY order_index ASC",
            track_id,
        )
        .fetch_all(&mut **tx.get())
        .await?;

        Ok(ids)
    }

    /// テーブル全体の件数を取得
    /// # todo
    /// テストでしか使ってない
    async fn count_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<u32> {
        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM track_artworks"#)
            .fetch_one(&mut **tx.get())
            .await?;

        Ok(count.try_into()?)
    }

    /// 指定されたアートワークIDのレコード数を取得
    async fn count_by_artwork_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        artwork_id: i32,
    ) -> Result<u32> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM track_artworks WHERE artwork_id = $1"#,
            artwork_id,
        )
        .fetch_one(&mut **tx.get())
        .await?;

        Ok(count.try_into()?)
    }

    /// 新規登録
    async fn insert<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        track_id: i32,
        order_index: usize,
        artwork_id: i32,
        picture_type: u8,
        description: &str,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO track_artworks (track_id, order_index, artwork_id, picture_type, description) VALUES($1, $2, $3, $4, $5)",
            track_id, order_index as i32, artwork_id, picture_type as i32, description,
        ).execute(&mut **tx.get()).await?;

        Ok(())
    }

    /// 曲IDを指定して削除
    async fn delete_by_song_id<'c>(&self, tx: &mut DbTransaction<'c>, track_id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM track_artworks WHERE track_id = $1", track_id,)
            .execute(&mut **tx.get())
            .await?;

        Ok(())
    }
}
