use super::{SongSyncEntry, SongSyncRow};
use crate::{converts::DbLibSongPathRef, sql_func};
use anyhow::Result;
use domain::{db_wrapper::TransactionWrapper, path::LibSongPath};
use mockall::automock;
use rusqlite::{Row, named_params, params};

/// SongSyncRowのDAO
#[automock]
pub trait SongSyncDao {
    /// パスを指定して取得
    fn select_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibSongPath,
    ) -> Result<Option<SongSyncRow>>;

    /// SongSyncデータを更新
    fn update<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
        entry: &SongSyncEntry<'c>,
    ) -> Result<()>;
}

/// SongSyncDaoの本実装
pub struct SongSyncDaoImpl {}

impl SongSyncDao for SongSyncDaoImpl {
    /// パスを指定して取得
    fn select_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibSongPath,
    ) -> Result<Option<SongSyncRow>> {
        let sql = format!("select {} from [song] where [path] = ?", ALL_COLUMNS);
        sql_func::select_opt(tx, &sql, params![DbLibSongPathRef::from(path)], map_all)
    }

    /// SongSyncデータを更新
    fn update<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
        entry: &SongSyncEntry<'c>,
    ) -> Result<()> {
        sql_func::execute(
            tx,
            "update [song] set [duration] = :duration, [title] = :title, [artist] = :artist, [album] = :album, [genre] = :genre, [album_artist] = :album_artist, [composer] = :composer, [track_number] = :track_number, [track_max] = :track_max, [disc_number] = :disc_number, [disc_max] = :disc_max, [release_date] = :release_date, [memo] = :memo, [lyrics] = :lyrics, [title_order] = :title_order, [artist_order] = :artist_order, [album_order] = :album_order, [album_artist_order] = :album_artist_order, [composer_order] = :composer_order, [genre_order] = :genre_order where [rowid] = :rowid",
            named_params! {
                ":duration": entry.duration,
                ":title": entry.title,
                ":artist": entry.artist,
                ":album": entry.album,
                ":genre": entry.genre,
                ":album_artist": entry.album_artist,
                ":composer": entry.composer,
                ":track_number": entry.track_number,
                ":track_max": entry.track_max,
                ":disc_number": entry.disc_number,
                ":disc_max": entry.disc_max,
                ":release_date": entry.release_date,
                ":memo": entry.memo,
                ":lyrics": entry.lyrics,
                ":title_order": entry.title_order,
                ":artist_order": entry.artist_order,
                ":album_order": entry.album_order,
                ":album_artist_order": entry.album_artist_order,
                ":composer_order": entry.composer_order,
                ":genre_order": entry.genre_order,
                ":rowid": song_id,
            },
        )
    }
}

/// 全カラム名
const ALL_COLUMNS: &str = "[rowid],[duration],[title],[artist],[album],[genre],[album_artist],[composer],[track_number],[track_max],[disc_number],[disc_max],[release_date],[memo],[lyrics]";

/// 全カラム取得時のマッパー
fn map_all(row: &Row) -> rusqlite::Result<SongSyncRow> {
    Ok(SongSyncRow {
        rowid: row.get(0)?,
        duration: row.get(1)?,
        title: row.get(2)?,
        artist: row.get(3)?,
        album: row.get(4)?,
        genre: row.get(5)?,
        album_artist: row.get(6)?,
        composer: row.get(7)?,
        track_number: row.get(8)?,
        track_max: row.get(9)?,
        disc_number: row.get(10)?,
        disc_max: row.get(11)?,
        release_date: row.get(12)?,
        memo: row.get(13)?,
        lyrics: row.get(14)?,
    })
}
