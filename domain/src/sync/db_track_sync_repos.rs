use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;
use sqlx::PgTransaction;

use crate::{
    folder::FolderIdMayRoot,
    path::LibTrackPath,
    sync::{DbTrackSync, TrackSync},
};

/// PCと連携するための曲データのリポジトリ
#[async_trait]
pub trait DbTrackSyncRepository {
    /// パスを指定して曲情報を取得
    ///
    /// # Arguments
    /// - path 曲のパス
    /// # Returns
    /// 該当する曲の情報（見つからない場合はNone）
    async fn get_by_path<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<Option<DbTrackSync>>;

    /// 曲を新規登録
    ///
    /// # Arguments
    /// - track_path: 追加する曲のパス
    /// - track_sync: 登録する曲のデータ
    /// - folder_id: 追加先のライブラリフォルダのID
    ///
    /// # Return
    /// 追加した曲のID
    async fn register<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track_path: &LibTrackPath,
        track_sync: &TrackSync,
        folder_id: FolderIdMayRoot,
    ) -> Result<i32>;

    /// 曲の連携情報をDBに保存(アートワーク以外)
    ///
    /// アートワークは重すぎるので除外。
    /// DbArtworkRepositoryの保存処理を直接呼び出すこと。
    async fn save_exclude_artwork<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track: &DbTrackSync,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbTrackSyncRepository {
    pub inner: MockDbTrackSyncRepositoryInner,
}
#[async_trait]
impl DbTrackSyncRepository for MockDbTrackSyncRepository {
    async fn get_by_path<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<Option<DbTrackSync>> {
        self.inner.get_by_path(path)
    }

    async fn register<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        track_path: &LibTrackPath,
        track_sync: &TrackSync,
        folder_id: FolderIdMayRoot,
    ) -> Result<i32> {
        self.inner.register(track_path, track_sync, folder_id)
    }

    async fn save_exclude_artwork<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        track: &DbTrackSync,
    ) -> Result<()> {
        self.inner.save_exclude_artwork(track)
    }
}
mock! {
    pub DbTrackSyncRepositoryInner {
        pub fn get_by_path(
            &self,
            path: &LibTrackPath,
        ) -> Result<Option<DbTrackSync>>;

        pub fn register(
            &self,
            track_path: &LibTrackPath,
            track_sync: &TrackSync,
            folder_id: FolderIdMayRoot,
        ) -> Result<i32>;

        pub fn save_exclude_artwork(
            &self,
            track: &DbTrackSync,
        ) -> Result<()>;
    }
}
