use anyhow::Result;
use sqlx::PgTransaction;

use crate::{
    db_utils::like_esc,
    path::{LibraryDirectoryPath, LibraryTrackPath},
};

/// ディレクトリを指定してパスを取得
/// # Arguments
/// - path: 検索対象のライブラリパス
/// # Returns
/// 指定されたディレクトリ内の、全ての曲のパス
pub async fn get_path_by_directory<'c>(
    tx: &mut PgTransaction<'c>,
    path: &LibraryDirectoryPath,
) -> Result<Vec<LibraryTrackPath>> {
    let path_str: &str = path.as_ref();

    //LIKE文エスケープ
    let cmp_value_buff;
    let (like_query, cmp_value) = if like_esc::is_need(path_str) {
        cmp_value_buff = like_esc::escape(path_str);
        ("LIKE $1 || '%' ESCAPE '$'", cmp_value_buff.as_str())
    } else {
        ("LIKE $1 || '%'", path_str)
    };

    let sql = format!("SELECT path FROM tracks WHERE path {like_query}");
    let paths = sqlx::query_scalar(&sql)
        .bind(cmp_value)
        .fetch_all(&mut **tx)
        .await?;

    Ok(paths)
}

#[cfg(test)]
mod tests;
