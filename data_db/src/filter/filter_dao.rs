use super::FilterRow;
use crate::sql_func;
use anyhow::Result;
use domain::db_wrapper::TransactionWrapper;
use mockall::automock;
use rusqlite::{params, Row};

/// filterテーブルのDAO
#[automock]
pub trait FilterDao {
    /// 最上位IDを指定して検索
    fn select_by_root_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        root_id: i32,
    ) -> Result<Vec<FilterRow>>;
}

/// FilterDaoの本実装
pub struct FilterDaoImpl;

impl FilterDao for FilterDaoImpl {
    /// 最上位IDを指定して検索
    ///
    /// in_parnet_order順
    fn select_by_root_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        root_id: i32,
    ) -> Result<Vec<FilterRow>> {
        let sql = format!(
            "select {} from [filter] where [root_id] = ? order by [in_parent_order]",
            ALL_COLUMNS
        );
        sql_func::select_list(tx, &sql, params![root_id], map_all)
    }
}

/// 全カラムの列名
const ALL_COLUMNS: &str =
    "[rowid],[parent_id],[in_parent_order],[root_id],[target],[str_value],[str_value2],[range]";

/// 全カラム取得時のマッパー
fn map_all(row: &Row) -> rusqlite::Result<FilterRow> {
    Ok(FilterRow {
        rowid: row.get(0)?,
        parent_id: row.get(1)?,
        in_parent_order: row.get(2)?,
        root_id: row.get(3)?,
        target: row.get(4)?,
        str_value: row.get(5)?,
        str_value2: row.get(6)?,
        range: row.get(7)?,
    })
}
