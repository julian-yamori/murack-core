use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use murack_core_domain::artwork::{DbArtworkRepository, TrackArtwork};
use murack_core_media::picture::Picture;
use sqlx::PgTransaction;

use super::{ArtworkCache, ArtworkCachedData, ArtworkImageRow};

/// DbArtworkRepositoryの本実装
#[derive(new)]
pub struct DbArtworkRepositoryImpl {
    artwork_cache: Arc<Mutex<ArtworkCache>>,
}

impl DbArtworkRepositoryImpl {
    /// アートワークを新規登録する
    ///
    /// 既に登録されていた場合は、新規登録せずに既存データのIDを返す
    ///
    /// # Arguments
    /// - picture: アートワークの画像データ
    /// # Return
    /// 新規登録されたアートワーク、もしくは既存の同一データのID
    async fn register_artwork(
        &self,
        tx: &mut PgTransaction<'_>,
        picture: &Arc<Picture>,
    ) -> Result<i32> {
        //対象データのMD5ハッシュを取得
        let hash = picture.hash();

        //追加用キャッシュと比較
        if let Some(ref c) = self.lock_cache()?.cache {
            if c.picture.bytes == picture.bytes {
                return Ok(c.artwork_id);
            }
        }

        //同じハッシュのデータをDBから検索
        let same_hash_list = sqlx::query_as!(
            ArtworkImageRow,
            "SELECT id, image, mime_type FROM artworks WHERE hash = $1",
            &hash[..]
        )
        .fetch_all(&mut **tx)
        .await?;
        //見つかった各データを走査
        for existing in same_hash_list {
            //データ本体の比較を行い、これも一致したら新規作成しない
            if picture.bytes == existing.image {
                //キャッシュにも保存
                self.lock_cache()?.cache = Some(ArtworkCachedData {
                    artwork_id: existing.id,
                    picture: Arc::new(Picture {
                        bytes: existing.image,
                        mime_type: existing.mime_type,
                    }),
                });
                return Ok(existing.id);
            }
        }

        //縮小画像を作成
        let image_mini = picture.artwork_mini_image()?;

        //新規追加を実行
        let new_pk = sqlx::query_scalar!(
            "INSERT INTO artworks (hash, image, image_mini, mime_type) values($1,$2,$3,$4) RETURNING id",
            &hash[..], &picture.bytes, &image_mini[..], &picture.mime_type
        ).fetch_one(&mut **tx).await?;

        //すぐに使う可能性が高いので、キャッシュに保存
        self.lock_cache()?.cache = Some(ArtworkCachedData {
            artwork_id: new_pk,
            //Arc Clone
            picture: picture.clone(),
        });

        Ok(new_pk)
    }

    fn lock_cache(&self) -> Result<MutexGuard<ArtworkCache>> {
        self.artwork_cache
            .lock()
            .map_err(|_| anyhow!("artwork cache lock error"))
    }
}

#[async_trait]
impl DbArtworkRepository for DbArtworkRepositoryImpl {
    /// 曲に紐づくアートワークの情報を取得する
    /// # Arguments
    /// - track_id: アートワーク情報を取得する曲のID
    /// # Returns
    /// 指定された曲に紐づく全アートワークの情報
    async fn get_track_artworks<'c>(
        &self,
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

        Ok(artworks.into_iter().collect::<anyhow::Result<_>>()?)
    }

    /// 曲にアートワークの紐付きを登録
    ///
    /// orderは無視し、関数内で上書きする。
    ///
    /// # Arguments
    /// - track_id: 紐付けを登録する曲のID
    /// - track_artworks: 曲に紐づく全てのアートワークの情報
    async fn register_track_artworks<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track_id: i32,
        track_artworks: &[TrackArtwork],
    ) -> Result<()> {
        //一旦、現在の紐付きを全て解除
        self.unregister_track_artworks(tx, track_id).await?;

        for (artwork_idx, artwork) in track_artworks.iter().enumerate() {
            //アートワーク画像情報を新規登録し、ID情報を取得
            let artwork_id = self.register_artwork(tx, &artwork.picture).await?;

            //紐付き情報を登録
            sqlx::query!(
                "INSERT INTO track_artworks (track_id, order_index, artwork_id, picture_type, description) VALUES($1, $2, $3, $4, $5)",
                track_id, artwork_idx as i32, artwork_id, artwork.picture_type as i32, artwork.description,
            ).execute(&mut **tx).await?;
        }

        Ok(())
    }

    /// 曲へのアートワーク紐付き情報を削除
    ///
    /// どの曲にも紐付かないアートワークは、DBから削除する
    ///
    /// # Arguments
    /// - track_id 紐付けを削除する曲のID
    async fn unregister_track_artworks<'c>(
        &self,
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
                sqlx::query!("DELETE FROM artworks WHERE id = $1", artwork_id,)
                    .execute(&mut **tx)
                    .await?;

                //キャッシュされていたら削除
                let mut artwork_cache = self.lock_cache()?;
                if let Some(ref c) = artwork_cache.cache {
                    if c.artwork_id == artwork_id {
                        artwork_cache.cache = None;
                    }
                }
            }
        }

        Ok(())
    }
}
