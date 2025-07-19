use anyhow::Result;
use domain::db::DbTransaction;

/// artworkテーブルのDAO
pub struct ArtworkDao {}

impl ArtworkDao {
    /// テーブル全体の件数を取得
    ///
    /// # todo テストでしか使ってない
    pub async fn count_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<u32> {
        let count: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM artworks"#)
            .fetch_one(&mut **tx.get())
            .await?;

        Ok(count.try_into()?)
    }

    /// 新規登録
    ///
    /// # Returns
    /// 登録されたレコードのrowid
    pub async fn insert<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        hash: &[u8],
        image: &[u8],
        image_mini: &[u8],
        mime_type: &str,
    ) -> Result<i32> {
        let id = sqlx::query_scalar!(
            "INSERT INTO artworks (hash, image, image_mini, mime_type) values($1,$2,$3,$4) RETURNING id",
            hash, image, image_mini, mime_type
        ).fetch_one(&mut **tx.get()).await?;

        Ok(id)
    }

    /// 1件削除
    pub async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, artwork_id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM artworks WHERE id = $1", artwork_id,)
            .execute(&mut **tx.get())
            .await?;

        Ok(())
    }
}
