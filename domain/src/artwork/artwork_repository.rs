use sqlx::PgTransaction;

use crate::artwork::{ArtworkError, ArtworkHash, MiniImage, Picture};

/// アートワークを DB に追加
pub async fn add_artwork(
    tx: &mut PgTransaction<'_>,
    picture: &Picture,
) -> Result<i32, ArtworkError> {
    //対象データのMD5ハッシュを取得
    let hash = ArtworkHash::from_image(&picture.bytes);

    //縮小画像を作成
    let image_mini = MiniImage::from_original_image(&picture.bytes)?;

    //新規追加を実行
    let artwork_id = sqlx::query_scalar!(
            "INSERT INTO artworks (hash, image, image_mini, mime_type) values($1,$2,$3,$4) RETURNING id",
            hash.as_ref(), &picture.bytes, image_mini.as_ref(), &picture.mime_type
        ).fetch_one(&mut **tx).await?;

    Ok(artwork_id)
}

/// アートワークを DB から削除
pub async fn delete_artwork(
    tx: &mut PgTransaction<'_>,
    artwork_id: i32,
) -> Result<(), ArtworkError> {
    sqlx::query!("DELETE FROM artworks WHERE id = $1", artwork_id,)
        .execute(&mut **tx)
        .await?;

    Ok(())
}
