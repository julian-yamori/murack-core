//! DB 操作の共通関数 (未整理)

use murack_core_domain::path::LibraryTrackPath;
use sqlx::PgPool;

use crate::track_sync::{TrackSync, track_sync_repository};

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
