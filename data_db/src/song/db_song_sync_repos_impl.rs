use super::{SongDao, SongEntry, SongSyncDao, SongSyncEntry};
use crate::converts::DbDate;
use anyhow::Result;
use chrono::NaiveDate;
use domain::{
    artwork::DbArtworkRepository,
    db_wrapper::TransactionWrapper,
    folder::FolderIdMayRoot,
    path::LibSongPath,
    sync::{DbSongSync, DbSongSyncRepository, SongSync},
};
use std::rc::Rc;

/// DbSongSyncRepositoryの本実装
#[derive(new)]
pub struct DbSongSyncRepositoryImpl {
    db_artwork_repository: Rc<dyn DbArtworkRepository>,
    song_dao: Rc<dyn SongDao>,
    song_sync_dao: Rc<dyn SongSyncDao>,
}

impl DbSongSyncRepository for DbSongSyncRepositoryImpl {
    /// パスを指定して曲情報を取得
    ///
    /// # Arguments
    /// - path 曲のパス
    /// # Returns
    /// 該当する曲の情報（見つからない場合はNone）
    fn get_by_path(
        &self,
        tx: &TransactionWrapper,
        path: &LibSongPath,
    ) -> Result<Option<DbSongSync>> {
        //一旦songテーブルから検索
        let song_row = match self.song_sync_dao.select_by_path(tx, path)? {
            Some(t) => t,
            None => return Ok(None),
        };

        Ok(Some(DbSongSync {
            id: song_row.rowid,
            path: path.clone(),
            song_sync: SongSync {
                duration: song_row.duration,
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
                release_date: song_row.release_date.map(NaiveDate::from),
                memo: song_row.memo.into(),
                lyrics: song_row.lyrics.into(),
                //アートワーク情報を検索して紐づけ
                artworks: self
                    .db_artwork_repository
                    .get_song_artworks(tx, song_row.rowid)?,
            },
        }))
    }

    /// 曲を新規登録
    ///
    /// # Arguments
    /// - song_path: 追加する曲のパス
    /// - song_sync: 登録する曲のデータ
    /// - folder_id: 追加先のライブラリフォルダのID
    /// - entry_date: 登録日
    ///
    /// # Return
    /// 追加した曲のID
    fn register(
        &self,
        tx: &TransactionWrapper,
        song_path: &LibSongPath,
        song_sync: &SongSync,
        folder_id: FolderIdMayRoot,
        entry_date: NaiveDate,
    ) -> Result<i32> {
        let song_dao = &self.song_dao;

        //DBに既に存在しないか確認
        //TODO unique keyにする
        if song_dao.exists_path(tx, song_path)? {
            return Err(domain::Error::DbSongAlreadyExists(song_path.clone()).into());
        }

        let song_id = song_dao.insert(
            tx,
            &SongEntry {
                duration: song_sync.duration,
                path: song_path.into(),
                folder_id: folder_id.into(),
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
                release_date: song_sync.release_date.map(DbDate::from),
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
                entry_date: entry_date.into(),
            },
        )?;

        //アートワークを登録
        self.db_artwork_repository
            .register_song_artworks(tx, song_id, &song_sync.artworks)?;

        Ok(song_id)
    }

    /// 曲の連携情報をDBに保存(アートワーク以外)
    ///
    /// アートワークは重すぎるので除外。
    /// ArtworkRepositoryの保存処理を直接呼び出すこと。
    fn save_exclude_artwork(&self, tx: &TransactionWrapper, song: &DbSongSync) -> Result<()> {
        let sync = &song.song_sync;

        self.song_sync_dao.update(
            tx,
            song.id,
            &SongSyncEntry {
                duration: sync.duration,
                title: sync.title.as_deref().unwrap_or_default(),
                artist: sync.artist.as_deref().unwrap_or_default(),
                album: sync.album.as_deref().unwrap_or_default(),
                genre: sync.genre.as_deref().unwrap_or_default(),
                album_artist: sync.album_artist.as_deref().unwrap_or_default(),
                composer: sync.composer.as_deref().unwrap_or_default(),
                track_number: sync.track_number,
                track_max: sync.track_max,
                disc_number: sync.disc_number,
                disc_max: sync.disc_max,
                release_date: sync.release_date.map(DbDate::from),
                memo: sync.memo.as_deref().unwrap_or_default(),
                lyrics: sync.lyrics.as_deref().unwrap_or_default(),
                title_order: sync.title_order().as_deref().unwrap_or_default(),
                artist_order: sync.artist_order().as_deref().unwrap_or_default(),
                album_order: sync.album_order().as_deref().unwrap_or_default(),
                album_artist_order: sync.album_artist_order().as_deref().unwrap_or_default(),
                composer_order: sync.composer_order().as_deref().unwrap_or_default(),
                genre_order: sync.genre_order().as_deref().unwrap_or_default(),
            },
        )?;

        Ok(())
    }
}
