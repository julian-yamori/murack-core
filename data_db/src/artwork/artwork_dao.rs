use crate::sql_func;
use anyhow::Result;
use domain::db_wrapper::TransactionWrapper;
use mockall::automock;
use rusqlite::params;

/// artworkテーブルのDAO
#[automock]
pub trait ArtworkDao {
    /// テーブル全体の件数を取得
    ///
    /// # todo テストでしか使ってない
    fn count_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<u32>;

    /// 新規登録
    ///
    /// # Returns
    /// 登録されたレコードのrowid
    fn insert<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        hash: &[u8],
        image: &[u8],
        image_mini: &[u8],
        mime_type: &str,
    ) -> Result<i32>;

    /// 1件削除
    fn delete<'c>(&self, tx: &TransactionWrapper<'c>, artwork_id: i32) -> Result<()>;
}

/// ArtworkDaoの本実装
pub struct ArtworkDaoImpl {}

impl ArtworkDao for ArtworkDaoImpl {
    /// テーブル全体の件数を取得
    ///
    /// # todo テストでしか使ってない
    fn count_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<u32> {
        sql_func::select_val(tx, "select count(*) from [artwork]", [])
    }

    /// 新規登録
    ///
    /// # Returns
    /// 登録されたレコードのrowid
    fn insert<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        hash: &[u8],
        image: &[u8],
        image_mini: &[u8],
        mime_type: &str,
    ) -> Result<i32> {
        sql_func::insert_get(
            tx,
            "insert into [artwork]([hash],[image],[image_mini],[mime_type]) values(?,?,?,?)",
            params![hash, image, image_mini, mime_type,],
        )
    }

    /// 1件削除
    fn delete<'c>(&self, tx: &TransactionWrapper<'c>, artwork_id: i32) -> Result<()> {
        sql_func::execute(
            tx,
            "delete from [artwork] where [rowid] = ?",
            params![artwork_id],
        )
    }
}
