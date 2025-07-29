#[cfg(test)]
mod tests;

use anyhow::Result;
use sqlx::PgTransaction;

use super::TrackSync;
use crate::{
    folder::{FolderIdMayRoot, folder_repository},
    path::LibraryTrackPath,
    playlist::playlist_repository,
    sync::track_sync_repository,
};

/// DBに曲データを新規登録する
///
/// # Arguments
/// - db: DB接続
/// - track_path: 登録する曲のライブラリ内パス
/// - track_sync: 登録する曲のデータ
pub async fn register_db<'c>(
    tx: &mut PgTransaction<'c>,
    track_path: &LibraryTrackPath,
    track_sync: &TrackSync,
) -> Result<()> {
    //親ディレクトリを登録してIDを取得
    let parent_path_opt = track_path.parent();
    let folder_id = match parent_path_opt {
        None => FolderIdMayRoot::Root,
        Some(parent_path) => {
            let id = folder_repository::register_not_exists(tx, &parent_path).await?;
            FolderIdMayRoot::Folder(id)
        }
    };

    //DBに書き込み
    track_sync_repository::register(tx, track_path, track_sync, folder_id).await?;

    //プレイリストのリストアップ済みフラグを解除
    playlist_repository::reset_listuped_flag(tx).await?;

    Ok(())
}
