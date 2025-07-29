#[cfg(test)]
mod tests;

use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;
use sqlx::PgTransaction;

use super::{DbTrackSyncRepository, TrackSync};
use crate::{
    folder::{FolderIdMayRoot, folder_repository},
    path::LibraryTrackPath,
    playlist::playlist_repository,
};

/// DB・PC連携のUseCase
#[async_trait]
pub trait SyncUsecase {
    /// DBに曲データを新規登録する
    ///
    /// # Arguments
    /// - db: DB接続
    /// - track_path: 登録する曲のライブラリ内パス
    /// - track_sync: 登録する曲のデータ
    async fn register_db<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track_path: &LibraryTrackPath,
        track_sync: &TrackSync,
    ) -> Result<()>;
}

/// SyncUsecaseの本実装
#[derive(new)]
pub struct SyncUsecaseImpl<SSR>
where
    SSR: DbTrackSyncRepository + Sync + Send,
{
    db_track_sync_repository: SSR,
}
#[async_trait]
impl<SSR> SyncUsecase for SyncUsecaseImpl<SSR>
where
    SSR: DbTrackSyncRepository + Sync + Send,
{
    /// DBに曲データを新規登録する
    ///
    /// # Arguments
    /// - db: DB接続
    /// - track_path: 登録する曲のライブラリ内パス
    /// - track_sync: 登録する曲のデータ
    async fn register_db<'c>(
        &self,
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
        self.db_track_sync_repository
            .register(tx, track_path, track_sync, folder_id)
            .await?;

        //プレイリストのリストアップ済みフラグを解除
        playlist_repository::reset_listuped_flag(tx).await?;

        Ok(())
    }
}

#[derive(Default)]
pub struct MockSyncUsecase {
    pub inner: MockSyncUsecaseInner,
}
#[async_trait]
impl SyncUsecase for MockSyncUsecase {
    async fn register_db<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        track_path: &LibraryTrackPath,
        track_sync: &TrackSync,
    ) -> Result<()> {
        self.inner.register_db(track_path, track_sync)
    }
}
mock! {
    pub SyncUsecaseInner {
        pub fn register_db(
            &self,
            track_path: &LibraryTrackPath,
            track_sync: &TrackSync,
        ) -> Result<()>;
    }
}
