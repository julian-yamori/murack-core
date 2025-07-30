use anyhow::Result;
use sqlx::PgTransaction;

use crate::{
    Error as DomainError,
    artwork::artwork_repository,
    db_utils::like_esc,
    folder::{FolderIdMayRoot, folder_repository},
    path::{LibraryDirectoryPath, LibraryTrackPath},
    playlist::{playlist_repository, playlist_track_repository},
    tag::track_tag_repository,
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

/// 曲のパスを書き換え
///
/// # Arguments
/// - old_path: 書き換え元の曲のパス
/// - new_path: 書き換え先の曲のパス
/// - new_folder_id: 新しい親フォルダのID
pub async fn update_path<'c>(
    tx: &mut PgTransaction<'c>,
    old_path: &LibraryTrackPath,
    new_path: &LibraryTrackPath,
    new_folder_id: FolderIdMayRoot,
) -> Result<()> {
    sqlx::query!(
        "UPDATE tracks SET path = $1, folder_id = $2 WHERE path = $3",
        new_path.as_ref() as &str,
        new_folder_id.into_db(),
        old_path.as_ref() as &str,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// 曲の再生時間を書き換え
pub async fn update_duration<'c>(
    tx: &mut PgTransaction<'c>,
    track_id: i32,
    duration: u32,
) -> Result<()> {
    let duration_i32: i32 = duration.try_into()?;

    sqlx::query!(
        "UPDATE tracks SET duration = $1 WHERE id = $2",
        duration_i32,
        track_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// DBから曲を削除
///
/// # Arguments
/// - path: 削除する曲のパス
pub async fn delete_track_db<'c>(
    tx: &mut PgTransaction<'c>,
    path: &LibraryTrackPath,
) -> Result<()> {
    // 指定されたパスの曲の ID を取得
    let track_id = sqlx::query_scalar!(
        "SELECT id FROM tracks WHERE path = $1",
        path.as_ref() as &str
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| DomainError::DbTrackNotFound(path.clone()))?;

    //曲の削除
    sqlx::query!("DELETE FROM tracks WHERE id = $1", track_id,)
        .execute(&mut **tx)
        .await?;

    //プレイリストからこの曲を削除
    playlist_track_repository::delete_track_from_all_playlists(tx, track_id).await?;

    //タグと曲の紐付けを削除
    track_tag_repository::delete_all_tags_from_track(tx, track_id).await?;

    //他に使用する曲がなければ、アートワークを削除
    artwork_repository::unregister_track_artworks(tx, track_id).await?;

    //他に使用する曲がなければ、親フォルダを削除
    if let Some(parent) = path.parent() {
        folder_repository::delete_db_if_empty(tx, &parent).await?;
    };

    playlist_repository::reset_listuped_flag(tx).await?;

    Ok(())
}

#[cfg(test)]
mod tests;
