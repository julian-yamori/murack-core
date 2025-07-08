use super::super::sql_func;
use anyhow::Result;
use domain::db_wrapper::TransactionWrapper;
use mockall::automock;
use rusqlite::params;

/// song_artworkテーブルのDAO
#[automock]
pub trait SongArtworkDao {
    /// 曲IDを指定してアートワークIDを取得
    ///
    /// order順
    fn select_artwork_id_by_song_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
    ) -> Result<Vec<i32>>;

    /// テーブル全体の件数を取得
    /// # todo
    /// テストでしか使ってない
    fn count_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<u32>;

    /// 指定されたアートワークIDのレコード数を取得
    fn count_by_artwork_id<'c>(&self, tx: &TransactionWrapper<'c>, artwork_id: i32) -> Result<u32>;

    /// 新規登録
    fn insert<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
        order: usize,
        artwork_id: i32,
        picture_type: u8,
        description: &str,
    ) -> Result<()>;

    /// 曲IDを指定して削除
    fn delete_by_song_id<'c>(&self, tx: &TransactionWrapper<'c>, song_id: i32) -> Result<()>;
}

/// SongArtworkDaoの本実装
pub struct SongArtworkDaoImpl {}

impl SongArtworkDao for SongArtworkDaoImpl {
    /// 曲IDを指定してアートワークIDを取得
    ///
    /// order順
    fn select_artwork_id_by_song_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
    ) -> Result<Vec<i32>> {
        sql_func::select_list(
            tx,
            "select [artwork_id] from [song_artwork] where [song_id] = ? order by [order] asc",
            params![song_id],
            |row| row.get(0),
        )
    }

    /// テーブル全体の件数を取得
    /// # todo
    /// テストでしか使ってない
    fn count_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<u32> {
        sql_func::select_val(tx, "select count(*) from [song_artwork]", [])
    }

    /// 指定されたアートワークIDのレコード数を取得
    fn count_by_artwork_id<'c>(&self, tx: &TransactionWrapper<'c>, artwork_id: i32) -> Result<u32> {
        sql_func::select_val(
            tx,
            "select count(*) from [song_artwork] where [artwork_id] = ?",
            params![artwork_id],
        )
    }

    /// 新規登録
    fn insert<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
        order: usize,
        artwork_id: i32,
        picture_type: u8,
        description: &str,
    ) -> Result<()> {
        sql_func::execute(tx, "insert into [song_artwork]([song_id],[order],[artwork_id],[picture_type],[description]) values(?,?,?,?,?)", params![song_id, order, artwork_id, picture_type, description])
    }

    /// 曲IDを指定して削除
    fn delete_by_song_id<'c>(&self, tx: &TransactionWrapper<'c>, song_id: i32) -> Result<()> {
        sql_func::execute(
            tx,
            "delete from [song_artwork] where [song_id] = ?",
            params![song_id],
        )
    }
}
