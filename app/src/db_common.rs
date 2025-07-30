//! DB 操作の共通関数 (未整理)

#[cfg(test)]
mod tests;

use murack_core_domain::{
    NonEmptyString,
    path::{LibraryDirectoryPath, LibraryTrackPath},
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
