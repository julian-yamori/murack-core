use anyhow::Result;
use async_trait::async_trait;
use domain::{db::DbTransaction, path::LibSongPath};

use super::{SongSyncEntry, SongSyncRow};

/// SongSyncRowのDAO
#[async_trait]
pub trait SongSyncDao {
    /// パスを指定して取得
    async fn select_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<SongSyncRow>>;

    /// SongSyncデータを更新
    async fn update<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        entry: &SongSyncEntry,
    ) -> Result<()>;
}

/// SongSyncDaoの本実装
pub struct SongSyncDaoImpl {}

#[async_trait]
impl SongSyncDao for SongSyncDaoImpl {
    /// パスを指定して取得
    async fn select_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<SongSyncRow>> {
        let row = sqlx::query_as!(SongSyncRow, "SELECT id, duration, title, artist, album, genre, album_artist, composer, track_number, track_max, disc_number, disc_max, release_date, memo, lyrics FROM tracks WHERE path = $1", path.as_str()).fetch_optional(&mut **tx.get()).await?;

        Ok(row)
    }

    /// SongSyncデータを更新
    async fn update<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        entry: &SongSyncEntry,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE tracks SET duration = $1, title = $2, artist = $3, album = $4, genre = $5, album_artist = $6, composer = $7, track_number = $8, track_max = $9, disc_number = $10, disc_max = $11, release_date = $12, memo = $13, lyrics = $14, title_order = $15, artist_order = $16, album_order = $17, album_artist_order = $18, composer_order = $19, genre_order = $20 WHERE id = $21",
            entry.duration,
            entry.title,
            entry.artist,
            entry.album,
            entry.genre,
            entry.album_artist,
            entry.composer,
            entry.track_number,
            entry.track_max,
            entry.disc_number,
            entry.disc_max,
            entry.release_date,
            entry.memo,
            entry.lyrics,
            entry.title_order,
            entry.artist_order,
            entry.album_order,
            entry.album_artist_order,
            entry.composer_order,
            entry.genre_order,
            song_id,
        ).execute(&mut **tx.get()).await?;

        Ok(())
    }
}
