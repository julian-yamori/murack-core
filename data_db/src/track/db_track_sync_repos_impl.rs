use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{
    Error as DomainError,
    artwork::DbArtworkRepository,
    folder::FolderIdMayRoot,
    path::LibTrackPath,
    sync::{DbTrackSync, DbTrackSyncRepository, TrackSync},
};
use sqlx::PgTransaction;

use crate::converts::enums::db_from_folder_id_may_root;

use super::{TrackSyncRow, track_sqls};

/// DbTrackSyncRepositoryの本実装
#[derive(new)]
pub struct DbTrackSyncRepositoryImpl<DAR>
where
    DAR: DbArtworkRepository + Sync + Send,
{
    db_artwork_repository: DAR,
}

#[async_trait]
impl<DAR> DbTrackSyncRepository for DbTrackSyncRepositoryImpl<DAR>
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
        tx: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<Option<DbTrackSync>> {
        //一旦trackテーブルから検索
        let track_row = match sqlx::query_as!(TrackSyncRow, "SELECT id, duration, title, artist, album, genre, album_artist, composer, track_number, track_max, disc_number, disc_max, release_date, memo, lyrics FROM tracks WHERE path = $1", path.as_str()).fetch_optional(&mut **tx).await? {
            Some(t) => t,
            None => return Ok(None),
        };

        Ok(Some(DbTrackSync {
            id: track_row.id,
            path: path.clone(),
            track_sync: TrackSync {
                duration: track_row.duration.try_into()?,
                title: track_row.title.into(),
                artist: track_row.artist.into(),
                album: track_row.album.into(),
                genre: track_row.genre.into(),
                album_artist: track_row.album_artist.into(),
                composer: track_row.composer.into(),
                track_number: track_row.track_number,
                track_max: track_row.track_max,
                disc_number: track_row.disc_number,
                disc_max: track_row.disc_max,
                release_date: track_row.release_date,
                memo: track_row.memo.into(),
                lyrics: track_row.lyrics.into(),
                //アートワーク情報を検索して紐づけ
                artworks: self
                    .db_artwork_repository
                    .get_track_artworks(tx, track_row.id)
                    .await?,
            },
        }))
    }

    /// 曲を新規登録
    ///
    /// # Arguments
    /// - track_path: 追加する曲のパス
    /// - track_sync: 登録する曲のデータ
    /// - folder_id: 追加先のライブラリフォルダのID
    ///
    /// # Return
    /// 追加した曲のID
    async fn register<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track_path: &LibTrackPath,
        track_sync: &TrackSync,
        folder_id: FolderIdMayRoot,
    ) -> Result<i32> {
        //DBに既に存在しないか確認
        //TODO unique keyにする
        if track_sqls::exists_path(tx, track_path).await? {
            return Err(DomainError::DbTrackAlreadyExists(track_path.clone()).into());
        }

        // order系の値を事前に計算しておく
        let title_order = track_sync.title_order().unwrap_or_default();
        let artist_order = track_sync.artist_order().unwrap_or_default();
        let album_order = track_sync.album_order().unwrap_or_default();
        let album_artist_order = track_sync.album_artist_order().unwrap_or_default();
        let composer_order = track_sync.composer_order().unwrap_or_default();
        let genre_order = track_sync.genre_order().unwrap_or_default();

        let track_id = sqlx::query_scalar!(
            "INSERT INTO tracks (duration, path, folder_id, title, artist, album, genre, album_artist, composer, track_number, track_max, disc_number, disc_max, release_date, rating, original_track, suggest_target, memo, memo_manage, lyrics, title_order, artist_order, album_order, album_artist_order, composer_order, genre_order) values($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26) RETURNING id",
            i32::try_from(track_sync.duration)?,
            track_path.as_str(),
            db_from_folder_id_may_root(folder_id),
            track_sync.title.as_deref().unwrap_or_default(),
            track_sync.artist.as_deref().unwrap_or_default(),
            track_sync.album.as_deref().unwrap_or_default(),
            track_sync.genre.as_deref().unwrap_or_default(),
            track_sync.album_artist.as_deref().unwrap_or_default(),
            track_sync.composer.as_deref().unwrap_or_default(),
            track_sync.track_number,
            track_sync.track_max,
            track_sync.disc_number,
            track_sync.disc_max,
            track_sync.release_date,
            0, // rating
            "", // original_track
            true, // suggest_target
            track_sync.memo.as_deref().unwrap_or_default(),
            "", // memo_manage,
            track_sync.lyrics.as_deref().unwrap_or_default(),
            title_order,
            artist_order,
            album_order,
            album_artist_order,
            composer_order,
            genre_order,
        ).fetch_one(&mut **tx).await?;

        //アートワークを登録
        self.db_artwork_repository
            .register_track_artworks(tx, track_id, &track_sync.artworks)
            .await?;

        Ok(track_id)
    }

    /// 曲の連携情報をDBに保存(アートワーク以外)
    ///
    /// アートワークは重すぎるので除外。
    /// ArtworkRepositoryの保存処理を直接呼び出すこと。
    async fn save_exclude_artwork<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track: &DbTrackSync,
    ) -> Result<()> {
        let sync = &track.track_sync;

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
            track.id,
        ).execute(&mut **tx).await?;

        Ok(())
    }
}
