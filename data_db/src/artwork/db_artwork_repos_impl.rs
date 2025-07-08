use super::{ArtworkCache, ArtworkCachedData, ArtworkDao, ArtworkImageDao, SongArtworkDao};
use crate::sql_func;
use anyhow::Result;
use domain::{
    artwork::{DbArtworkRepository, SongArtwork},
    db_wrapper::TransactionWrapper,
};
use media::picture::Picture;
use rusqlite::params;
use std::{cell::RefCell, rc::Rc};

/// DbArtworkRepositoryの本実装
#[derive(new)]
pub struct DbArtworkRepositoryImpl {
    artwork_cache: Rc<RefCell<ArtworkCache>>,
    artwork_dao: Rc<dyn ArtworkDao>,
    artwork_image_dao: Rc<dyn ArtworkImageDao>,
    song_artwork_dao: Rc<dyn SongArtworkDao>,
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
    fn register_artwork(&self, tx: &TransactionWrapper, picture: &Rc<Picture>) -> Result<i32> {
        //対象データのMD5ハッシュを取得
        let hash = picture.hash();

        //追加用キャッシュと比較
        if let Some(ref c) = self.artwork_cache.borrow_mut().cache {
            if c.picture.bytes == picture.bytes {
                return Ok(c.artwork_id);
            }
        }

        //同じハッシュのデータをDBから検索
        let same_hash_list = self.artwork_image_dao.select_by_hash(tx, &hash[..])?;
        //見つかった各データを走査
        for existing in same_hash_list {
            //データ本体の比較を行い、これも一致したら新規作成しない
            if picture.bytes == existing.image {
                //キャッシュにも保存
                self.artwork_cache.borrow_mut().cache = Some(ArtworkCachedData {
                    artwork_id: existing.rowid,
                    picture: Rc::new(Picture {
                        bytes: existing.image,
                        mime_type: existing.mime_type,
                    }),
                });
                return Ok(existing.rowid);
            }
        }

        //縮小画像を作成
        let image_mini = picture.artwork_mini_image()?;

        //新規追加を実行
        let new_pk = self.artwork_dao.insert(
            tx,
            &hash[..],
            &picture.bytes,
            &image_mini[..],
            &picture.mime_type,
        )?;

        //すぐに使う可能性が高いので、キャッシュに保存
        self.artwork_cache.borrow_mut().cache = Some(ArtworkCachedData {
            artwork_id: new_pk,
            //Rc Clone
            picture: picture.clone(),
        });

        Ok(new_pk)
    }
}

impl DbArtworkRepository for DbArtworkRepositoryImpl {
    /// 曲に紐づくアートワークの情報を取得する
    /// # Arguments
    /// - song_id: アートワーク情報を取得する曲のID
    /// # Returns
    /// 指定された曲に紐づく全アートワークの情報
    fn get_song_artworks(&self, tx: &TransactionWrapper, song_id: i32) -> Result<Vec<SongArtwork>> {
        sql_func::select_list(
            tx,
            "select a.[image], a.[mime_type], sa.[picture_type], sa.[description], sa.[order]
                from [song_artwork] as sa
                left join [artwork] as a
                    on sa.[artwork_id] = a.[rowid]
                where sa.[song_id] = ?
                order by [order] asc",
            params![song_id],
            |row| {
                Ok(SongArtwork {
                    picture: Rc::new(Picture {
                        bytes: row.get(0)?,
                        mime_type: row.get(1)?,
                    }),
                    picture_type: row.get(2)?,
                    description: row.get(3)?,
                })
            },
        )
    }

    /// 曲にアートワークの紐付きを登録
    ///
    /// orderは無視し、関数内で上書きする。
    ///
    /// # Arguments
    /// - song_id: 紐付けを登録する曲のID
    /// - song_artworks: 曲に紐づく全てのアートワークの情報
    fn register_song_artworks(
        &self,
        tx: &TransactionWrapper,
        song_id: i32,
        song_artworks: &[SongArtwork],
    ) -> Result<()> {
        //一旦、現在の紐付きを全て解除
        self.unregister_song_artworks(tx, song_id)?;

        for (artwork_idx, artwork) in song_artworks.iter().enumerate() {
            //アートワーク画像情報を新規登録し、ID情報を取得
            let artwork_id = self.register_artwork(tx, &artwork.picture)?;

            //紐付き情報を登録
            self.song_artwork_dao.insert(
                tx,
                song_id,
                artwork_idx,
                artwork_id,
                artwork.picture_type,
                &artwork.description,
            )?;
        }

        Ok(())
    }

    /// 曲へのアートワーク紐付き情報を削除
    ///
    /// どの曲にも紐付かないアートワークは、DBから削除する
    ///
    /// # Arguments
    /// - song_id 紐付けを削除する曲のID
    fn unregister_song_artworks(&self, tx: &TransactionWrapper, song_id: i32) -> Result<()> {
        //今紐付いているアートワークのIDを取得
        let artwork_ids = self
            .song_artwork_dao
            .select_artwork_id_by_song_id(tx, song_id)?;

        //紐付きを解除
        self.song_artwork_dao.delete_by_song_id(tx, song_id)?;

        for artwork_id in artwork_ids {
            //他に紐付いている曲がなければ、このアートワークを削除
            let use_count = self.song_artwork_dao.count_by_artwork_id(tx, artwork_id)?;
            if use_count == 0 {
                self.artwork_dao.delete(tx, artwork_id)?;

                //キャッシュされていたら削除
                let mut artwork_cache = self.artwork_cache.borrow_mut();
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
