use super::FolderPathRow;
use crate::{
    converts::{DbFolderIdMayRoot, DbLibDirPathRef},
    sql_func,
};
use anyhow::Result;
use domain::{db_wrapper::TransactionWrapper, folder::FolderIdMayRoot, path::LibDirPath};
use mockall::automock;
use rusqlite::{params, Row};

/// folder_pathテーブルのDAO
#[automock]
pub trait FolderPathDao {
    /// IDを指定して検索
    fn select_by_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderPathRow>>;

    /// パスを指定して検索
    fn select_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
    ) -> Result<Option<FolderPathRow>>;

    /// パスを指定し、IDを取得
    fn select_id_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>>;

    /// 全レコード数を取得
    fn count_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<u32>;

    /// 親フォルダIDを指定してレコード数を取得
    fn count_by_parent_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        parent_id: FolderIdMayRoot,
    ) -> Result<u32>;

    /// 指定されたpathのレコードが存在するか確認
    fn exists_path<'c>(&self, tx: &TransactionWrapper<'c>, path: &LibDirPath) -> Result<bool>;

    /// 新規登録
    ///
    /// # Return
    /// 登録されたレコードのrowid
    fn insert<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
        name: &str,
        parent_id: FolderIdMayRoot,
    ) -> Result<i32>;

    /// IDを指定してフォルダを削除
    fn delete_by_id<'c>(&self, tx: &TransactionWrapper<'c>, folder_id: i32) -> Result<()>;
}

/// FolderPathDaoの本実装
pub struct FolderPathDaoImpl {}

impl FolderPathDao for FolderPathDaoImpl {
    /// IDを指定して検索
    fn select_by_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderPathRow>> {
        let sql = format!("select {} from [folder_path] where [id] = ?", ALL_COLUMNS);
        sql_func::select_opt(tx, &sql, params![folder_id], map_all)
    }

    /// パスを指定して検索
    fn select_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
    ) -> Result<Option<FolderPathRow>> {
        let sql = format!("select {} from [folder_path] where [path] = ?", ALL_COLUMNS);
        sql_func::select_opt(tx, &sql, params![DbLibDirPathRef::from(path)], map_all)
    }

    /// パスを指定し、IDを取得
    fn select_id_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>> {
        let sql = "select [id] from [folder_path] where [path] = ?";
        sql_func::select_opt(tx, sql, params![DbLibDirPathRef::from(path)], |row| {
            row.get(0)
        })
    }

    /// 全レコード数を取得
    fn count_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<u32> {
        sql_func::select_val(tx, "select count(*) from [folder_path]", [])
    }

    /// 親フォルダIDを指定してレコード数を取得
    fn count_by_parent_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        parent_id: FolderIdMayRoot,
    ) -> Result<u32> {
        sql_func::select_val(
            tx,
            "select count(*) from folder_path where [parent_id] is ?",
            params![DbFolderIdMayRoot::from(parent_id)],
        )
    }

    /// 指定されたpathのレコードが存在するか確認
    fn exists_path<'c>(&self, tx: &TransactionWrapper<'c>, path: &LibDirPath) -> Result<bool> {
        let count: u32 = sql_func::select_val(
            tx,
            "select count(*) from [folder_path] where [path] = ?",
            params![DbLibDirPathRef::from(path)],
        )?;
        Ok(count > 0)
    }

    /// 新規登録
    ///
    /// # Return
    /// 登録されたレコードのrowid
    fn insert<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
        name: &str,
        parent_id: FolderIdMayRoot,
    ) -> Result<i32> {
        //新規登録するIDを取得(autoincrementではないので手動で)
        let sql = "select COALESCE(max([id]),0) from [folder_path]";
        let before_id: i32 = sql_func::select_val(tx, sql, [])?;
        let next_id = before_id + 1;

        //このフォルダのパス情報を登録
        sql_func::execute(
            tx,
            "insert into [folder_path]([id],[path],[name],[parent_id]) values(?,?,?,?)",
            params![
                next_id,
                DbLibDirPathRef::from(path),
                name,
                DbFolderIdMayRoot::from(parent_id)
            ],
        )?;

        Ok(next_id)
    }

    /// IDを指定してフォルダを削除
    fn delete_by_id<'c>(&self, tx: &TransactionWrapper<'c>, folder_id: i32) -> Result<()> {
        sql_func::execute(
            tx,
            "delete from [folder_path] where [id] = ?",
            params![folder_id],
        )
    }
}

/// 全カラムの列名
const ALL_COLUMNS: &str = "[id],[path],[name],[parent_id]";

//全カラム取得時のマッパー
fn map_all(row: &Row) -> rusqlite::Result<FolderPathRow> {
    Ok(FolderPathRow {
        rowid: row.get(0)?,
        path: row.get(1)?,
        name: row.get(2)?,
        parent_id: row.get(3)?,
    })
}
