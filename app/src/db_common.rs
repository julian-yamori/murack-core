//! DB 操作の共通関数 (未整理)

use murack_core_domain::{
    path::LibraryTrackPath,
    sync::{SyncUsecase, TrackSync},
};
use sqlx::PgPool;

/// 曲を DB に新規登録
pub async fn add_track_to_db<SYS>(
    db_pool: &PgPool,
    sync_usecase: &SYS,
    track_path: &LibraryTrackPath,
    track_sync: &mut TrackSync,
) -> anyhow::Result<()>
where
    SYS: SyncUsecase + Send + Sync,
{
    //曲名が空なら、ファイル名から取得
    if track_sync.title.is_empty() {
        track_sync.title = track_path.file_stem().to_owned();
    };

    let mut tx = db_pool.begin().await?;

    sync_usecase
        .register_db(&mut tx, track_path, track_sync)
        .await?;

    tx.commit().await?;
    Ok(())
}
