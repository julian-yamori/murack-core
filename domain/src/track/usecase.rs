use anyhow::Result;

use crate::{NonEmptyString, path::LibraryTrackPath, track::track_repository};
use sqlx::PgTransaction;

/// パス文字列を指定してDBから削除
///
/// # Arguments
/// - path: 削除する曲のパス
///
/// # Returns
/// 削除した曲のパスリスト
pub async fn delete_path_str_db<'c>(
    tx: &mut PgTransaction<'c>,
    path_str: &NonEmptyString,
) -> Result<Vec<LibraryTrackPath>> {
    let track_path_list = track_repository::get_path_by_path_str(tx, path_str).await?;

    for path in &track_path_list {
        track_repository::delete_track_db(tx, path).await?;
    }

    Ok(track_path_list)
}
