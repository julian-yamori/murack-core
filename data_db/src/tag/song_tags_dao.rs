use crate::sql_func;
use anyhow::Result;
use domain::db_wrapper::TransactionWrapper;
use mockall::automock;
use rusqlite::params;

/// song_tagsテーブルのDAO
#[automock]
pub trait SongTagsDao {
    /// レコードを新規登録
    fn insert<'c>(&self, tx: &TransactionWrapper<'c>, song_id: i32, tag_id: i32) -> Result<()>;

    /// 曲IDでレコードを削除
    fn delete_by_song_id<'c>(&self, tx: &TransactionWrapper<'c>, song_id: i32) -> Result<()>;
}

/// SongTagsDaoの本実装
pub struct SongTagsDaoImpl {}

impl SongTagsDao for SongTagsDaoImpl {
    /// レコードを新規登録
    fn insert<'c>(&self, tx: &TransactionWrapper<'c>, song_id: i32, tag_id: i32) -> Result<()> {
        sql_func::execute(
            tx,
            "insert into [song_tags]([song_id], [tag_id]) values(?, ?)",
            params![song_id, tag_id],
        )
    }

    /// 曲IDでレコードを削除
    fn delete_by_song_id<'c>(&self, tx: &TransactionWrapper<'c>, song_id: i32) -> Result<()> {
        sql_func::execute(
            tx,
            "delete from [song_tags] where [song_id] = ?",
            params![song_id],
        )
    }
}
