use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use domain::{
    artwork::{DbArtworkRepository, SongArtwork},
    db::DbTransaction,
};
use media::picture::Picture;

use super::{ArtworkCache, ArtworkCachedData, ArtworkDao, ArtworkImageDao, SongArtworkDao};

/// DbArtworkRepositoryの本実装
#[derive(new)]
pub struct DbArtworkRepositoryImpl<AID, SAD>
where
    AID: ArtworkImageDao + Sync + Send,
    SAD: SongArtworkDao + Sync + Send,
{
    artwork_cache: Arc<Mutex<ArtworkCache>>,
    artwork_dao: ArtworkDao,
    artwork_image_dao: AID,
    song_artwork_dao: SAD,
}

impl<AID, SAD> DbArtworkRepositoryImpl<AID, SAD>
where
    AID: ArtworkImageDao + Sync + Send,
    SAD: SongArtworkDao + Sync + Send,
{
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
        tx: &mut DbTransaction<'_>,
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
        let same_hash_list = self.artwork_image_dao.select_by_hash(tx, &hash[..]).await?;
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
        let new_pk = self
            .artwork_dao
            .insert(
                tx,
                &hash[..],
                &picture.bytes,
                &image_mini[..],
                &picture.mime_type,
            )
            .await?;

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
impl<AID, SAD> DbArtworkRepository for DbArtworkRepositoryImpl<AID, SAD>
where
    AID: ArtworkImageDao + Sync + Send,
    SAD: SongArtworkDao + Sync + Send,
{
    /// 曲に紐づくアートワークの情報を取得する
    /// # Arguments
    /// - song_id: アートワーク情報を取得する曲のID
    /// # Returns
    /// 指定された曲に紐づく全アートワークの情報
    async fn get_song_artworks<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        track_id: i32,
    ) -> Result<Vec<SongArtwork>> {
        let artworks = sqlx::query!(
            "SELECT a.image, a.mime_type, sa.picture_type, sa.description, sa.order_index
                FROM track_artworks as sa
                LEFT JOIN artworks as a
                    ON sa.artwork_id = a.id
                WHERE sa.track_id = $1
                ORDER BY order_index ASC",
            track_id,
        )
        .map(|row| -> anyhow::Result<SongArtwork> {
            Ok(SongArtwork {
                picture: Arc::new(Picture {
                    bytes: row.image,
                    mime_type: row.mime_type,
                }),
                picture_type: row.picture_type.try_into()?,
                description: row.description,
            })
        })
        .fetch_all(&mut **tx.get())
        .await?;

        Ok(artworks.into_iter().collect::<anyhow::Result<_>>()?)
    }

    /// 曲にアートワークの紐付きを登録
    ///
    /// orderは無視し、関数内で上書きする。
    ///
    /// # Arguments
    /// - song_id: 紐付けを登録する曲のID
    /// - song_artworks: 曲に紐づく全てのアートワークの情報
    async fn register_song_artworks<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        song_artworks: &[SongArtwork],
    ) -> Result<()> {
        //一旦、現在の紐付きを全て解除
        self.unregister_song_artworks(tx, song_id).await?;

        for (artwork_idx, artwork) in song_artworks.iter().enumerate() {
            //アートワーク画像情報を新規登録し、ID情報を取得
            let artwork_id = self.register_artwork(tx, &artwork.picture).await?;

            //紐付き情報を登録
            self.song_artwork_dao
                .insert(
                    tx,
                    song_id,
                    artwork_idx,
                    artwork_id,
                    artwork.picture_type,
                    &artwork.description,
                )
                .await?;
        }

        Ok(())
    }

    /// 曲へのアートワーク紐付き情報を削除
    ///
    /// どの曲にも紐付かないアートワークは、DBから削除する
    ///
    /// # Arguments
    /// - song_id 紐付けを削除する曲のID
    async fn unregister_song_artworks<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()> {
        //今紐付いているアートワークのIDを取得
        let artwork_ids = self
            .song_artwork_dao
            .select_artwork_id_by_song_id(tx, song_id)
            .await?;

        //紐付きを解除
        self.song_artwork_dao.delete_by_song_id(tx, song_id).await?;

        for artwork_id in artwork_ids {
            //他に紐付いている曲がなければ、このアートワークを削除
            let use_count = self
                .song_artwork_dao
                .count_by_artwork_id(tx, artwork_id)
                .await?;
            if use_count == 0 {
                self.artwork_dao.delete(tx, artwork_id).await?;

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
