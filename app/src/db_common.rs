//! DB 操作の共通関数 (未整理)

#[cfg(test)]
mod tests;

use murack_core_domain::{
    NonEmptyString,
    folder::folder_repository,
    path::{LibraryDirectoryPath, LibraryTrackPath},
    playlist::{playlist_sqls, playlist_tracks_sqls},
    track::track_sqls,
};
use sqlx::{PgPool, PgTransaction};

use crate::{
    DbTrackError, app_artwork_repository, track_data::AudioMetadata,
    track_sync::track_sync_repository,
};

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
    let mut list = track_sqls::get_path_by_directory(tx, &dir_path).await?;

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
    mut metadata: AudioMetadata,
) -> anyhow::Result<()> {
    //曲名が空なら、ファイル名から取得
    if metadata.title.is_empty() {
        metadata.title = track_path.file_stem().to_owned();
    };

    let mut tx = db_pool.begin().await?;

    track_sync_repository::register_db(&mut tx, track_path, metadata).await?;

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
    .ok_or_else(|| DbTrackError::DbTrackNotFound(path.clone()))?;

    //曲の削除
    sqlx::query!("DELETE FROM tracks WHERE id = $1", track_id,)
        .execute(&mut **tx)
        .await?;

    //プレイリストからこの曲を削除
    delete_track_from_all_playlists(tx, track_id).await?;

    //タグと曲の紐付けを削除
    sqlx::query!("DELETE FROM track_tags WHERE track_id = $1", track_id,)
        .execute(&mut **tx)
        .await?;

    //他に使用する曲がなければ、アートワークを削除
    app_artwork_repository::unregister_track_artworks(tx, track_id).await?;

    //他に使用する曲がなければ、親フォルダを削除
    if let Some(parent) = path.parent() {
        folder_repository::delete_if_empty(tx, &parent).await?;
    };

    playlist_sqls::reset_listuped_flag(tx).await?;

    Ok(())
}

//曲を全プレイリストから削除
async fn delete_track_from_all_playlists<'c>(
    tx: &mut PgTransaction<'c>,
    track_id: i32,
) -> anyhow::Result<()> {
    // 全てのプレイリストについて処理
    let playlist_ids = sqlx::query_scalar!(r#"SELECT id FROM playlists"#)
        .fetch_all(&mut **tx)
        .await?;
    for playlist_id in playlist_ids {
        //プレイリスト内の曲を取得
        let tracks = playlist_tracks_sqls::select_track_id_by_playlist_id(tx, playlist_id).await?;

        //プレイリストから一旦全削除
        playlist_tracks_sqls::delete_by_playlist_id(tx, playlist_id).await?;

        //削除対象の曲を除き、全て追加
        let add_tracks = tracks.iter().filter(|i| **i != track_id).enumerate();
        for (order, it) in add_tracks {
            playlist_tracks_sqls::insert_playlist_track(tx, playlist_id, *it, order as i32).await?;
        }
    }

    Ok(())
}
