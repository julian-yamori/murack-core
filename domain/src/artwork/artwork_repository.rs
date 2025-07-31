use std::sync::Arc;

use murack_core_media::picture::Picture;
use sqlx::PgTransaction;

/// アートワークを DB に追加
pub async fn add_artwork(
    tx: &mut PgTransaction<'_>,
    picture: &Arc<Picture>,
) -> anyhow::Result<i32> {
    //対象データのMD5ハッシュを取得
    let hash = picture.hash();

    //縮小画像を作成
    let image_mini = picture.artwork_mini_image()?;

    //新規追加を実行
    let artwork_id = sqlx::query_scalar!(
            "INSERT INTO artworks (hash, image, image_mini, mime_type) values($1,$2,$3,$4) RETURNING id",
            &hash[..], &picture.bytes, &image_mini[..], &picture.mime_type
        ).fetch_one(&mut **tx).await?;

    Ok(artwork_id)
}

/// アートワークを DB から削除
pub async fn delete_artwork(tx: &mut PgTransaction<'_>, artwork_id: i32) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM artworks WHERE id = $1", artwork_id,)
        .execute(&mut **tx)
        .await?;

    Ok(())
}
