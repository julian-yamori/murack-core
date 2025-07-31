use std::sync::Arc;

use anyhow::Result;
use murack_core_media::picture::Picture;
use sqlx::PgTransaction;

use crate::artwork::TrackArtwork;

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

/// 曲に紐づくアートワークの情報を取得する
/// # Arguments
/// - track_id: アートワーク情報を取得する曲のID
/// # Returns
/// 指定された曲に紐づく全アートワークの情報
pub async fn get_track_artworks<'c>(
    tx: &mut PgTransaction<'c>,
    track_id: i32,
) -> Result<Vec<TrackArtwork>> {
    let artworks = sqlx::query!(
        "SELECT a.image, a.mime_type, sa.picture_type, sa.description, sa.order_index
                FROM track_artworks as sa
                LEFT JOIN artworks as a
                    ON sa.artwork_id = a.id
                WHERE sa.track_id = $1
                ORDER BY order_index ASC",
        track_id,
    )
    .map(|row| -> anyhow::Result<TrackArtwork> {
        Ok(TrackArtwork {
            picture: Arc::new(Picture {
                bytes: row.image,
                mime_type: row.mime_type,
            }),
            picture_type: row.picture_type.try_into()?,
            description: row.description,
        })
    })
    .fetch_all(&mut **tx)
    .await?;

    artworks.into_iter().collect::<anyhow::Result<_>>()
}

/// アートワークを DB から削除
pub async fn delete_artwork(tx: &mut PgTransaction<'_>, artwork_id: i32) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM artworks WHERE id = $1", artwork_id,)
        .execute(&mut **tx)
        .await?;

    Ok(())
}
