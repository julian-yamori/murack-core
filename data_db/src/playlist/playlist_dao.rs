use super::PlaylistRow;
use crate::sql_func;
use anyhow::Result;
use domain::db_wrapper::TransactionWrapper;
use mockall::automock;
use rusqlite::{Row, params};

/// playlistテーブルのDAO
#[automock]
pub trait PlaylistDao {
    /// IDを指定して検索
    fn select_by_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        plist_id: i32,
    ) -> Result<Option<PlaylistRow>>;

    /// 全レコードを取得
    fn select_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<Vec<PlaylistRow>>;

    /// 全レコードを取得(in_folder_order順)
    fn select_all_order_folder<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<Vec<PlaylistRow>>;

    /// プレイリストの子プレイリスト一覧を取得
    /// # Arguments
    /// - parent_id: 親プレイリストID(Noneなら最上位のプレイリストを取得)
    /// # Returns
    /// 指定されたプレイリストの子プレイリスト一覧
    fn get_child_playlists<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        parent_id: Option<i32>,
    ) -> Result<Vec<PlaylistRow>>;
}

/// PlaylistDaoの本実装
pub struct PlaylistDaoImpl {}

impl PlaylistDao for PlaylistDaoImpl {
    /// IDを指定して検索
    fn select_by_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        plist_id: i32,
    ) -> Result<Option<PlaylistRow>> {
        let sql = format!("select {} from [playlist] where [rowid] = ?", ALL_COLUMNS);
        sql_func::select_opt(tx, &sql, params![plist_id], map_all)
    }

    /// 全レコードを取得
    fn select_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<Vec<PlaylistRow>> {
        let sql = format!("select {} from [playlist]", ALL_COLUMNS);
        sql_func::select_list(tx, &sql, [], map_all)
    }

    /// 全レコードを取得(in_folder_order順)
    fn select_all_order_folder<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<Vec<PlaylistRow>> {
        let sql = format!(
            "select {} from [playlist] order by [in_folder_order]",
            ALL_COLUMNS
        );
        sql_func::select_list(tx, &sql, [], map_all)
    }

    /// プレイリストの子プレイリスト一覧を取得
    /// # Arguments
    /// - parent_id: 親プレイリストID(Noneなら最上位のプレイリストを取得)
    /// # Returns
    /// 指定されたプレイリストの子プレイリスト一覧
    fn get_child_playlists<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        parent_id: Option<i32>,
    ) -> Result<Vec<PlaylistRow>> {
        let sql = format!(
            "select {} from [playlist] where [parent_id] is ? order by [in_folder_order]",
            ALL_COLUMNS
        );

        sql_func::select_list(tx, &sql, params![parent_id], map_all)
    }
}

/// 全カラム名
const ALL_COLUMNS: &str = "[rowid],[type],[name],[parent_id],[in_folder_order],[filter_root],[sort_type],[sort_desc],[save_dap],[listuped_flag],[dap_changed]";

/// 全カラム取得時のマッパー
fn map_all(row: &Row) -> rusqlite::Result<PlaylistRow> {
    Ok(PlaylistRow {
        rowid: row.get(0)?,
        playlist_type: row.get(1)?,
        name: row.get(2)?,
        parent_id: row.get(3)?,
        in_folder_order: row.get(4)?,
        filter_root_id: row.get(5)?,
        sort_type: row.get(6)?,
        sort_desc: row.get(7)?,
        save_dap: row.get(8)?,
        listuped_flag: row.get(9)?,
        dap_changed: row.get(10)?,
    })
}
