use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use crate::{
    db::DbTransaction,
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibPathStr, LibSongPath},
};

/// 曲データのDBリポジトリ
#[async_trait]
pub trait DbSongRepository {
    /// パスから曲IDを取得
    async fn get_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<i32>>;

    /// 文字列でパスを指定して、該当曲のパスリストを取得
    async fn get_path_by_path_str<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibPathStr,
    ) -> Result<Vec<LibSongPath>>;

    /// ディレクトリを指定してパスを取得
    /// # Arguments
    /// - path: 検索対象のライブラリパス
    /// # Returns
    /// 指定されたディレクトリ内の、全ての曲のパス
    async fn get_path_by_directory<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>>;

    /// ライブラリ内の全ての曲のパスを取得
    async fn get_path_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<Vec<LibSongPath>>;

    /// 指定したパスの曲が存在するか確認
    async fn is_exist_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<bool>;

    /// 指定されたフォルダに曲が存在するか確認
    async fn is_exist_in_folder<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<bool>;

    /// 曲のパスを書き換え
    ///
    /// # Arguments
    /// - old_path: 書き換え元の曲のパス
    /// - new_path: 書き換え先の曲のパス
    /// - new_folder_id: 新しい親フォルダのID
    async fn update_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()>;

    /// 曲の再生時間を書き換え
    async fn update_duration<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()>;

    /// 曲を削除
    ///
    /// # Arguments
    /// - song_id: 削除する曲のID
    async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, song_id: i32) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbSongRepository {
    pub inner: MockDbSongRepositoryInner,
}
#[async_trait]
impl DbSongRepository for MockDbSongRepository {
    async fn get_id_by_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<i32>> {
        self.inner.get_id_by_path(path)
    }

    async fn get_path_by_path_str<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibPathStr,
    ) -> Result<Vec<LibSongPath>> {
        self.inner.get_path_by_path_str(path)
    }

    async fn get_path_by_directory<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>> {
        self.inner.get_path_by_directory(path)
    }

    async fn get_path_all<'c>(&self, _db: &mut DbTransaction<'c>) -> Result<Vec<LibSongPath>> {
        self.inner.get_path_all()
    }

    async fn is_exist_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<bool> {
        self.inner.is_exist_path(path)
    }

    async fn is_exist_in_folder<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<bool> {
        self.inner.is_exist_in_folder(folder_id)
    }

    async fn update_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()> {
        self.inner.update_path(old_path, new_path, new_folder_id)
    }

    async fn update_duration<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()> {
        self.inner.update_duration(song_id, duration)
    }

    async fn delete<'c>(&self, _db: &mut DbTransaction<'c>, song_id: i32) -> Result<()> {
        self.inner.delete(song_id)
    }
}
mock! {
    pub DbSongRepositoryInner {
        pub fn get_id_by_path(
            &self,
            path: &LibSongPath,
        ) -> Result<Option<i32>>;

        pub fn get_path_by_path_str(
            &self,
            path: &LibPathStr,
        ) -> Result<Vec<LibSongPath>>;

        pub fn get_path_by_directory(
            &self,
            path: &LibDirPath,
        ) -> Result<Vec<LibSongPath>>;

        pub fn get_path_all(&self) -> Result<Vec<LibSongPath>>;

        pub fn is_exist_path(&self, path: &LibSongPath) -> Result<bool>;

        pub fn is_exist_in_folder(&self, folder_id: i32) -> Result<bool>;

        pub fn update_path(
            &self,
            old_path: &LibSongPath,
            new_path: &LibSongPath,
            new_folder_id: FolderIdMayRoot,
        ) -> Result<()>;

        pub fn update_duration(
            &self,
            song_id: i32,
            duration: u32,
        ) -> Result<()>;

        pub fn delete(&self, song_id: i32) -> Result<()>;
    }
}
