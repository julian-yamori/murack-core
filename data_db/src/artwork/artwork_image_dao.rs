use anyhow::Result;
use async_trait::async_trait;
use domain::db::DbTransaction;

use super::ArtworkImageRow;

/// ArtworkImageRowのDAO
#[async_trait]
pub trait ArtworkImageDao {
    /// ハッシュ値を指定して検索
    async fn select_by_hash<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        hash: &[u8],
    ) -> Result<Vec<ArtworkImageRow>>;
}

/// ArtworkImageDaoの本実装
pub struct ArtworkImageDaoImpl {}

#[async_trait]
impl ArtworkImageDao for ArtworkImageDaoImpl {
    /// ハッシュ値を指定して検索
    async fn select_by_hash<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        hash: &[u8],
    ) -> Result<Vec<ArtworkImageRow>> {
        let rows = sqlx::query_as!(
            ArtworkImageRow,
            "SELECT id, image, mime_type FROM artworks WHERE hash = $1",
            hash
        )
        .fetch_all(&mut **tx.get())
        .await?;

        Ok(rows)
    }
}
