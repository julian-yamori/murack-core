use super::super::sql_func;
use anyhow::Result;
use domain::db_wrapper::TransactionWrapper;
use mockall::automock;
use rusqlite::params;

/// playlist_songテーブルのDAO
#[automock]
pub trait PlaylistSongDao {
    /// プレイリストIDを指定して曲IDを取得
    fn select_song_id_by_playlist_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        plist_id: i32,
    ) -> Result<Vec<i32>>;

    /// 新規登録
    fn insert<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        plist_id: i32,
        song_id: i32,
        order: i32,
    ) -> Result<()>;

    /// プレイリストIDを指定して削除
    ///
    /// # Arguments
    /// - plist_id: 削除元のプレイリストのID
    fn delete_by_playlist_id<'c>(&self, tx: &TransactionWrapper<'c>, plist_id: i32) -> Result<()>;
}

/// PlaylistSongDaoの本実装
pub struct PlaylistSongDaoImpl {}

impl PlaylistSongDao for PlaylistSongDaoImpl {
    /// プレイリストIDを指定して曲IDを取得
    fn select_song_id_by_playlist_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        plist_id: i32,
    ) -> Result<Vec<i32>> {
        sql_func::select_list(
            tx,
            "select [song_id] from [playlist_song] where [playlist_id] = ?",
            params![plist_id],
            |row| row.get(0),
        )
    }

    /// 新規登録
    fn insert<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        plist_id: i32,
        song_id: i32,
        order: i32,
    ) -> Result<()> {
        sql_func::execute(
            tx,
            "insert into [playlist_song]([playlist_id],[order],[song_id]) values(?,?,?)",
            params![plist_id, order, song_id],
        )
    }

    /// プレイリストIDを指定して削除
    ///
    /// # Arguments
    /// - plist_id: 削除元のプレイリストのID
    fn delete_by_playlist_id<'c>(&self, tx: &TransactionWrapper<'c>, plist_id: i32) -> Result<()> {
        sql_func::execute(
            tx,
            "delete from [playlist_song] where [playlist_id] = ?",
            params![plist_id],
        )
    }
}
