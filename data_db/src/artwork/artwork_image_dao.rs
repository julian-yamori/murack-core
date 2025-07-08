use super::{super::sql_func, ArtworkImageRow};
use anyhow::Result;
use domain::db_wrapper::TransactionWrapper;
use mockall::automock;
use rusqlite::{params, Row};

/// ArtworkImageRowのDAO
#[automock]
pub trait ArtworkImageDao {
    /// ハッシュ値を指定して検索
    fn select_by_hash<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        hash: &[u8],
    ) -> Result<Vec<ArtworkImageRow>>;
}

/// ArtworkImageDaoの本実装
pub struct ArtworkImageDaoImpl {}

impl ArtworkImageDao for ArtworkImageDaoImpl {
    /// ハッシュ値を指定して検索
    fn select_by_hash<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        hash: &[u8],
    ) -> Result<Vec<ArtworkImageRow>> {
        let sql = format!("select {} from [artwork] where [hash] = ?", ALL_COLUMNS);
        sql_func::select_list(tx, &sql, params![hash], map_all)
    }
}

/// 全カラム名
const ALL_COLUMNS: &str = "[rowid],[image],[mime_type]";

//全カラムのマッパー
fn map_all(row: &Row) -> rusqlite::Result<ArtworkImageRow> {
    Ok(ArtworkImageRow {
        rowid: row.get(0)?,
        image: row.get(1)?,
        mime_type: row.get(2)?,
    })
}
