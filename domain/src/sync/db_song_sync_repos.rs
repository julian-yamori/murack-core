use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use crate::{
    db::DbTransaction,
    folder::FolderIdMayRoot,
    path::LibSongPath,
    sync::{DbSongSync, SongSync},
};

/// PCと連携するための曲データのリポジトリ
#[async_trait]
pub trait DbSongSyncRepository {
    /// パスを指定して曲情報を取得
    ///
    /// # Arguments
    /// - path 曲のパス
    /// # Returns
    /// 該当する曲の情報（見つからない場合はNone）
    async fn get_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<DbSongSync>>;

    /// 曲を新規登録
    ///
    /// # Arguments
    /// - song_path: 追加する曲のパス
    /// - song_sync: 登録する曲のデータ
    /// - folder_id: 追加先のライブラリフォルダのID
    ///
    /// # Return
    /// 追加した曲のID
    async fn register<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_path: &LibSongPath,
        song_sync: &SongSync,
        folder_id: FolderIdMayRoot,
    ) -> Result<i32>;

    /// 曲の連携情報をDBに保存(アートワーク以外)
    ///
    /// アートワークは重すぎるので除外。
    /// DbArtworkRepositoryの保存処理を直接呼び出すこと。
    async fn save_exclude_artwork<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song: &DbSongSync,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbSongSyncRepository {
    pub inner: MockDbSongSyncRepositoryInner,
}
#[async_trait]
impl DbSongSyncRepository for MockDbSongSyncRepository {
    async fn get_by_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<DbSongSync>> {
        self.inner.get_by_path(path)
    }

    async fn register<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        song_path: &LibSongPath,
        song_sync: &SongSync,
        folder_id: FolderIdMayRoot,
    ) -> Result<i32> {
        self.inner.register(song_path, song_sync, folder_id)
    }

    async fn save_exclude_artwork<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        song: &DbSongSync,
    ) -> Result<()> {
        self.inner.save_exclude_artwork(song)
    }
}
mock! {
    pub DbSongSyncRepositoryInner {
        pub fn get_by_path(
            &self,
            path: &LibSongPath,
        ) -> Result<Option<DbSongSync>>;

        pub fn register(
            &self,
            song_path: &LibSongPath,
            song_sync: &SongSync,
            folder_id: FolderIdMayRoot,
        ) -> Result<i32>;

        pub fn save_exclude_artwork(
            &self,
            song: &DbSongSync,
        ) -> Result<()>;
    }
}
