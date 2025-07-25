use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{
    Error as DomainError,
    artwork::DbArtworkRepository,
    db::DbTransaction,
    folder::FolderIdMayRoot,
    path::LibSongPath,
    sync::{DbSongSync, DbSongSyncRepository, SongSync},
};

use crate::converts::enums::db_from_folder_id_may_root;

use super::{SongEntry, SongSyncRow, song_sqls};

/// DbSongSyncRepositoryの本実装
#[derive(new)]
pub struct DbSongSyncRepositoryImpl<DAR>
where
    DAR: DbArtworkRepository + Sync + Send,
{
    db_artwork_repository: DAR,
}

#[async_trait]
impl<DAR> DbSongSyncRepository for DbSongSyncRepositoryImpl<DAR>
where
    DAR: DbArtworkRepository + Sync + Send,
{
    /// パスを指定して曲情報を取得
    ///
    /// # Arguments
    /// - path 曲のパス
    /// # Returns
    /// 該当する曲の情報（見つからない場合はNone）
    async fn get_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<DbSongSync>> {
        //一旦songテーブルから検索
        let song_row = match sqlx::query_as!(SongSyncRow, "SELECT id, duration, title, artist, album, genre, album_artist, composer, track_number, track_max, disc_number, disc_max, release_date, memo, lyrics FROM tracks WHERE path = $1", path.as_str()).fetch_optional(&mut **tx.get()).await? {
            Some(t) => t,
            None => return Ok(None),
        };

        Ok(Some(DbSongSync {
            id: song_row.id,
            path: path.clone(),
            song_sync: SongSync {
                duration: song_row.duration.try_into()?,
                title: song_row.title.into(),
                artist: song_row.artist.into(),
                album: song_row.album.into(),
                genre: song_row.genre.into(),
                album_artist: song_row.album_artist.into(),
                composer: song_row.composer.into(),
                track_number: song_row.track_number,
                track_max: song_row.track_max,
                disc_number: song_row.disc_number,
                disc_max: song_row.disc_max,
                release_date: song_row.release_date,
                memo: song_row.memo.into(),
                lyrics: song_row.lyrics.into(),
                //アートワーク情報を検索して紐づけ
                artworks: self
                    .db_artwork_repository
                    .get_song_artworks(tx, song_row.id)
                    .await?,
            },
        }))
    }

    /// 曲を新規登録
    ///
    /// # Arguments
    /// - song_path: 追加する曲のパス
    /// - song_sync: 登録する曲のデータ
    /// - folder_id: 追加先のライブラリフォルダのID
    ///
    /// # Return
    /// 追加した曲のID
    async fn register<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_path: &LibSongPath,
        song_sync: &SongSync,
        folder_id: FolderIdMayRoot,
    ) -> Result<i32> {
        //DBに既に存在しないか確認
        //TODO unique keyにする
        if song_sqls::exists_path(tx, song_path).await? {
            return Err(DomainError::DbSongAlreadyExists(song_path.clone()).into());
        }

        let song_id = song_sqls::insert(
            tx,
            &SongEntry {
                duration: song_sync.duration.try_into()?,
                path: song_path.as_str(),
                folder_id: db_from_folder_id_may_root(folder_id),
                title: song_sync.title.as_deref().unwrap_or_default(),
                artist: song_sync.artist.as_deref().unwrap_or_default(),
                album: song_sync.album.as_deref().unwrap_or_default(),
                genre: song_sync.genre.as_deref().unwrap_or_default(),
                album_artist: song_sync.album_artist.as_deref().unwrap_or_default(),
                composer: song_sync.composer.as_deref().unwrap_or_default(),
                track_number: song_sync.track_number,
                track_max: song_sync.track_max,
                disc_number: song_sync.disc_number,
                disc_max: song_sync.disc_max,
                release_date: song_sync.release_date,
                rating: 0,
                original_song: "",
                suggest_target: true,
                memo: song_sync.memo.as_deref().unwrap_or_default(),
                memo_manage: "",
                lyrics: song_sync.lyrics.as_deref().unwrap_or_default(),
                title_order: song_sync.title_order().as_deref().unwrap_or_default(),
                artist_order: song_sync.artist_order().as_deref().unwrap_or_default(),
                album_order: song_sync.album_order().as_deref().unwrap_or_default(),
                album_artist_order: song_sync
                    .album_artist_order()
                    .as_deref()
                    .unwrap_or_default(),
                composer_order: song_sync.composer_order().as_deref().unwrap_or_default(),
                genre_order: song_sync.genre_order().as_deref().unwrap_or_default(),
            },
        )
        .await?;

        //アートワークを登録
        self.db_artwork_repository
            .register_song_artworks(tx, song_id, &song_sync.artworks)
            .await?;

        Ok(song_id)
    }

    /// 曲の連携情報をDBに保存(アートワーク以外)
    ///
    /// アートワークは重すぎるので除外。
    /// ArtworkRepositoryの保存処理を直接呼び出すこと。
    async fn save_exclude_artwork<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song: &DbSongSync,
    ) -> Result<()> {
        let sync = &song.song_sync;

        // order系の値を事前に計算しておく
        let title_order = sync.title_order().unwrap_or_default();
        let artist_order = sync.artist_order().unwrap_or_default();
        let album_order = sync.album_order().unwrap_or_default();
        let album_artist_order = sync.album_artist_order().unwrap_or_default();
        let composer_order = sync.composer_order().unwrap_or_default();
        let genre_order = sync.genre_order().unwrap_or_default();

        // duration を i32 に変換
        let duration: i32 = sync.duration.try_into()?;

        sqlx::query!(
            "UPDATE tracks SET duration = $1, title = $2, artist = $3, album = $4, genre = $5, album_artist = $6, composer = $7, track_number = $8, track_max = $9, disc_number = $10, disc_max = $11, release_date = $12, memo = $13, lyrics = $14, title_order = $15, artist_order = $16, album_order = $17, album_artist_order = $18, composer_order = $19, genre_order = $20 WHERE id = $21",
            duration,
            sync.title.as_deref().unwrap_or_default(),
            sync.artist.as_deref().unwrap_or_default(),
            sync.album.as_deref().unwrap_or_default(),
            sync.genre.as_deref().unwrap_or_default(),
            sync.album_artist.as_deref().unwrap_or_default(),
            sync.composer.as_deref().unwrap_or_default(),
            sync.track_number,
            sync.track_max,
            sync.disc_number,
            sync.disc_max,
            sync.release_date,
            sync.memo.as_deref().unwrap_or_default(),
            sync.lyrics.as_deref().unwrap_or_default(),
            &title_order,
            &artist_order,
            &album_order,
            &album_artist_order,
            &composer_order,
            &genre_order,
            song.id,
        ).execute(&mut **tx.get()).await?;

        Ok(())
    }
}
