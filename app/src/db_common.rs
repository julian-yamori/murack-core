//! DB 操作の共通関数 (未整理)

#[cfg(test)]
mod tests;

use murack_core_domain::{
    Error as DomainError, NonEmptyString,
    artwork::artwork_repository,
    folder::folder_repository,
    path::{LibraryDirectoryPath, LibraryTrackPath},
    playlist::{playlist_repository, playlist_track_repository},
    tag::track_tag_repository,
    track::track_repository,
};
use sqlx::{PgPool, PgTransaction};

use crate::track_sync::{TrackSync, track_sync_repository};

/// 指定されたpathのレコードが存在するか確認
pub async fn exists_path<'c>(
    tx: &mut PgTransaction<'c>,
    path: &LibraryTrackPath,
) -> sqlx::Result<bool> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE path = $1"#,
        path.as_ref() as &str,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(count > 0)
}

/// 文字列でパスを指定して、該当曲のパスリストを取得
pub async fn track_paths_by_path_str<'c>(
    tx: &mut PgTransaction<'c>,
    path: &NonEmptyString,
) -> anyhow::Result<Vec<LibraryTrackPath>> {
    //ディレクトリ指定とみなして検索
    let dir_path: LibraryDirectoryPath = path.clone().into();
    let mut list = track_repository::get_path_by_directory(tx, &dir_path).await?;

    //ファイル指定とみなしての検索でヒットしたら追加
    let track_path: LibraryTrackPath = path.clone().into();
    if exists_path(tx, &track_path).await? {
        list.push(track_path);
    }

    Ok(list)
}

/// 曲を DB に新規登録
pub async fn add_track_to_db(
    db_pool: &PgPool,
    track_path: &LibraryTrackPath,
    track_sync: &mut TrackSync,
) -> anyhow::Result<()> {
    //曲名が空なら、ファイル名から取得
    if track_sync.title.is_empty() {
        track_sync.title = track_path.file_stem().to_owned();
    };

    let mut tx = db_pool.begin().await?;

    track_sync_repository::register_db(&mut tx, track_path, track_sync).await?;

    tx.commit().await?;
    Ok(())
}

/// DBから曲を削除
///
/// # Arguments
/// - path: 削除する曲のパス
pub async fn delete_track_db<'c>(
    tx: &mut PgTransaction<'c>,
    path: &LibraryTrackPath,
) -> anyhow::Result<()> {
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
