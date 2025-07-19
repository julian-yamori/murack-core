use anyhow::Result;
use async_trait::async_trait;
use domain::{
    artwork::DbArtworkRepository,
    db::DbTransaction,
    folder::FolderIdMayRoot,
    path::LibSongPath,
    sync::{DbSongSync, DbSongSyncRepository, SongSync},
};

use crate::converts::enums::db_from_folder_id_may_root;

use super::{SongDao, SongEntry, SongSyncDao, SongSyncEntry};

/// DbSongSyncRepositoryの本実装
#[derive(new)]
pub struct DbSongSyncRepositoryImpl<DAR, SD, SSD>
where
    DAR: DbArtworkRepository + Sync + Send,
    SD: SongDao + Sync + Send,
    SSD: SongSyncDao + Sync + Send,
{
    db_artwork_repository: DAR,
    song_dao: SD,
    song_sync_dao: SSD,
}

#[async_trait]
impl<DAR, SD, SSD> DbSongSyncRepository for DbSongSyncRepositoryImpl<DAR, SD, SSD>
where
    DAR: DbArtworkRepository + Sync + Send,
    SD: SongDao + Sync + Send,
    SSD: SongSyncDao + Sync + Send,
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
        let song_row = match self.song_sync_dao.select_by_path(tx, path).await? {
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
        let song_dao = &self.song_dao;

        //DBに既に存在しないか確認
        //TODO unique keyにする
        if song_dao.exists_path(tx, song_path).await? {
            return Err(domain::Error::DbSongAlreadyExists(song_path.clone()).into());
        }

        let song_id = song_dao
            .insert(
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

        self.song_sync_dao
            .update(
                tx,
                song.id,
                &SongSyncEntry {
                    duration: sync.duration.try_into()?,
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
                    release_date: sync.release_date,
                    memo: sync.memo.as_deref().unwrap_or_default(),
                    lyrics: sync.lyrics.as_deref().unwrap_or_default(),
                    title_order: sync.title_order().as_deref().unwrap_or_default(),
                    artist_order: sync.artist_order().as_deref().unwrap_or_default(),
                    album_order: sync.album_order().as_deref().unwrap_or_default(),
                    album_artist_order: sync.album_artist_order().as_deref().unwrap_or_default(),
                    composer_order: sync.composer_order().as_deref().unwrap_or_default(),
                    genre_order: sync.genre_order().as_deref().unwrap_or_default(),
                },
            )
            .await?;

        Ok(())
    }
}
