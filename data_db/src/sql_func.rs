//! rusqliteによるSQL文実行の共通関数

use anyhow::Result;
use domain::db_wrapper::TransactionWrapper;
use rusqlite::{types::FromSql, OptionalExtension, Params, Row};

/// selectで1行を取得し、Optionで返す
///
/// # Arguments
/// - tx: sqliteトランザクション
/// - sql: SQL文字列
/// - params: SQLパラメータ
/// - map: RowからTへのマッパー
pub fn select_opt<T>(
    tx: &TransactionWrapper,
    sql: &str,
    params: impl Params,
    f: impl FnOnce(&'_ Row) -> rusqlite::Result<T>,
) -> Result<Option<T>> {
    let rusq_tx = tx.get();
    let mut stmt = rusq_tx.prepare(sql)?;
    Ok(stmt.query_row(params, f).optional()?)
}

/// selectで複数行を取得
///
/// # Arguments
/// - tx: sqliteトランザクション
/// - sql: SQL文字列
/// - params: SQLパラメータ
/// - map: RowからTへのマッパー
pub fn select_list<T>(
    tx: &TransactionWrapper,
    sql: &str,
    params: impl Params,
    f: impl FnMut(&'_ Row) -> rusqlite::Result<T>,
) -> Result<Vec<T>> {
    let rusq_tx = tx.get();
    let mut stmt = rusq_tx.prepare(sql)?;
    let it = stmt.query_map(params, f)?;
    let r: rusqlite::Result<Vec<T>> = it.collect();
    Ok(r?)
}

/// selectで一つの値を取得(集計用)
///
/// # Arguments
/// - tx: sqliteトランザクション
/// - sql: SQL文字列
/// - params: SQLパラメータ
pub fn select_val<T>(tx: &TransactionWrapper, sql: &str, params: impl Params) -> Result<T>
where
    T: FromSql,
{
    let rusq_tx = tx.get();
    let mut stmt = rusq_tx.prepare(sql)?;
    Ok(stmt.query_row(params, |row| row.get(0))?)
}

/// SQLを単純実行
///
/// # Arguments
/// - tx: sqliteトランザクション
/// - sql: SQL文字列
/// - params: SQLパラメータ
pub fn execute(tx: &TransactionWrapper, sql: &str, params: impl Params) -> Result<()> {
    let rusq_tx = tx.get();
    rusq_tx.execute(sql, params)?;
    Ok(())
}

/// insertを実行後、追加したデータのrowidを取得する
///
/// # Arguments
/// - tx: sqliteトランザクション
/// - sql: SQL文字列
/// - table_name: 登録先のテーブル名
/// - params: SQLパラメータ
/// # Return
/// 追加したデータのrowid
pub fn insert_get(tx: &TransactionWrapper, sql: &str, params: impl Params) -> Result<i32> {
    let rusq_tx = tx.get();

    //通常の実行処理
    rusq_tx.execute(sql, params)?;

    Ok(rusq_tx.last_insert_rowid() as i32)
}
