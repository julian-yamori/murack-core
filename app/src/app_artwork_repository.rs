//! Murack Sync での DB のアートワーク操作

mod artwork_cache;

use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::{Result, anyhow};
use murack_core_domain::artwork::{ArtworkHash, TrackArtwork, artwork_repository};
use once_cell::sync::Lazy;
use sqlx::PgTransaction;

use crate::app_artwork_repository::artwork_cache::{ArtworkCache, ArtworkCachedData};

static ARTWORK_CACHE: Lazy<Arc<Mutex<ArtworkCache>>> =
    Lazy::new(|| Arc::new(Mutex::new(ArtworkCache::new())));

/// アートワークを新規登録する
///
/// 既に登録されていた場合は、新規登録せずに既存データのIDを返す
///
/// # Return
/// 新規登録されたアートワーク、もしくは既存の同一データのID
async fn register_artwork(
    tx: &mut PgTransaction<'_>,
    image: Vec<u8>,
    mime_type: &str,
) -> Result<i32> {
    //追加用キャッシュと比較
    if let Some(ref c) = lock_cache()?.cache {
        if c.image == image {
            return Ok(c.artwork_id);
        }
    }

    //同じハッシュのデータをDBから検索
    let hash = ArtworkHash::from_image(&image);
    let same_hash_list = sqlx::query!(
        "SELECT id, image FROM artworks WHERE hash = $1",
        hash.as_ref()
    )
    .fetch_all(&mut **tx)
    .await?;
    //見つかった各データを走査
    for existing in same_hash_list {
        //データ本体の比較を行い、これも一致したら新規作成しない
        if image == existing.image {
            //キャッシュにも保存
            lock_cache()?.cache = Some(ArtworkCachedData {
                artwork_id: existing.id,
                image: existing.image,
            });
            return Ok(existing.id);
        }
    }

    let new_pk = artwork_repository::add_artwork(tx, &image, mime_type).await?;

    //すぐに使う可能性が高いので、キャッシュに保存
    lock_cache()?.cache = Some(ArtworkCachedData {
        artwork_id: new_pk,
        image,
    });

    Ok(new_pk)
}

fn lock_cache() -> Result<MutexGuard<'static, ArtworkCache>> {
    ARTWORK_CACHE
        .lock()
        .map_err(|_| anyhow!("artwork cache lock error"))
}

/// 曲にアートワークの紐付きを登録
///
/// orderは無視し、関数内で上書きする。
///
/// # Arguments
/// - track_id: 紐付けを登録する曲のID
/// - track_artworks: 曲に紐づく全てのアートワークの情報
pub async fn register_track_artworks<'c>(
    tx: &mut PgTransaction<'c>,
    track_id: i32,
    track_artworks: Vec<TrackArtwork>,
) -> Result<()> {
    //一旦、現在の紐付きを全て解除
    unregister_track_artworks(tx, track_id).await?;

    for (artwork_idx, artwork) in track_artworks.into_iter().enumerate() {
        //アートワーク画像情報を新規登録し、ID情報を取得
        let artwork_id = register_artwork(tx, artwork.image, &artwork.mime_type).await?;

        //紐付き情報を登録
        sqlx::query!(
                "INSERT INTO track_artworks (track_id, order_index, artwork_id, picture_type, description) VALUES($1, $2, $3, $4, $5)",
                track_id, artwork_idx as i32, artwork_id, artwork.picture_type as i32, artwork.description,
            ).execute(&mut **tx).await?;
    }

    Ok(())
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
            image: row.image,
            mime_type: row.mime_type,
            picture_type: row.picture_type.try_into()?,
            description: row.description,
        })
    })
    .fetch_all(&mut **tx)
    .await?;

    artworks.into_iter().collect::<anyhow::Result<_>>()
}

/// 曲へのアートワーク紐付き情報を削除
///
/// どの曲にも紐付かないアートワークは、DBから削除する
///
/// # Arguments
/// - track_id 紐付けを削除する曲のID
pub async fn unregister_track_artworks<'c>(
    tx: &mut PgTransaction<'c>,
    track_id: i32,
) -> Result<()> {
    //今紐付いているアートワークのIDを取得
    let artwork_ids = sqlx::query_scalar!(
        "SELECT artwork_id FROM track_artworks WHERE track_id = $1 ORDER BY order_index ASC",
        track_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    //紐付きを解除
    sqlx::query!("DELETE FROM track_artworks WHERE track_id = $1", track_id,)
        .execute(&mut **tx)
        .await?;

    for artwork_id in artwork_ids {
        //他に紐付いている曲がなければ、このアートワークを削除
        let use_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM track_artworks WHERE artwork_id = $1"#,
            artwork_id,
        )
        .fetch_one(&mut **tx)
        .await?;
        if use_count == 0 {
            artwork_repository::delete_artwork(tx, artwork_id).await?;

            //キャッシュされていたら削除
            let mut artwork_cache = lock_cache()?;
            if let Some(ref c) = artwork_cache.cache {
                if c.artwork_id == artwork_id {
                    artwork_cache.cache = None;
                }
            }
        }
    }

    Ok(())
}
