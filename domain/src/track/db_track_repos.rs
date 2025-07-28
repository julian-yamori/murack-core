use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;
use sqlx::PgTransaction;

use crate::{
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibPathStr, LibTrackPath},
};

/// 曲データのDBリポジトリ
#[async_trait]
pub trait DbTrackRepository {
    /// パスから曲IDを取得
    async fn get_id_by_path<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<Option<i32>>;

    /// 文字列でパスを指定して、該当曲のパスリストを取得
    async fn get_path_by_path_str<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibPathStr,
    ) -> Result<Vec<LibTrackPath>>;

    /// 全ての曲のパスを取得
    async fn get_all_path<'c>(&self, tx: &mut PgTransaction<'c>) -> Result<Vec<LibTrackPath>>;

    /// ディレクトリを指定してパスを取得
    /// # Arguments
    /// - path: 検索対象のライブラリパス
    /// # Returns
    /// 指定されたディレクトリ内の、全ての曲のパス
    async fn get_path_by_directory<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibTrackPath>>;

    /// LibPathStr を曲ファイルのパスとみなすことができるか確認
    ///
    /// 曲のパスとみなすことができるなら `Some(LibTrackPath)` を返す。
    ///
    /// LibTrackPath への変換に失敗した (空文字列だった) 場合は None を返す。
    /// 曲ファイルがそのパスに存在しなかった場合も None を返す。
    async fn path_str_as_track_path<'c>(
        &self,
        tx: &mut PgTransaction,
        path_str: &LibPathStr,
    ) -> Result<Option<LibTrackPath>>;

    /// 指定したパスの曲が存在するか確認
    async fn is_exist_path<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<bool>;

    /// 指定されたフォルダに曲が存在するか確認
    async fn is_exist_in_folder<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
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
        tx: &mut PgTransaction<'c>,
        old_path: &LibTrackPath,
        new_path: &LibTrackPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()>;

    /// 曲の再生時間を書き換え
    async fn update_duration<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track_id: i32,
        duration: u32,
    ) -> Result<()>;

    /// 曲を削除
    ///
    /// # Arguments
    /// - track_id: 削除する曲のID
    async fn delete<'c>(&self, tx: &mut PgTransaction<'c>, track_id: i32) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbTrackRepository {
    pub inner: MockDbTrackRepositoryInner,
}
#[async_trait]
impl DbTrackRepository for MockDbTrackRepository {
    async fn get_id_by_path<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<Option<i32>> {
        self.inner.get_id_by_path(path)
    }

    async fn get_path_by_path_str<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        path: &LibPathStr,
    ) -> Result<Vec<LibTrackPath>> {
        self.inner.get_path_by_path_str(path)
    }

    async fn get_all_path<'c>(&self, _db: &mut PgTransaction<'c>) -> Result<Vec<LibTrackPath>> {
        self.inner.get_all_path()
    }

    async fn get_path_by_directory<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibTrackPath>> {
        self.inner.get_path_by_directory(path)
    }

    async fn path_str_as_track_path<'c>(
        &self,
        _db: &mut PgTransaction,
        path_str: &LibPathStr,
    ) -> Result<Option<LibTrackPath>> {
        self.inner.path_str_as_track_path(path_str)
    }

    async fn is_exist_path<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<bool> {
        self.inner.is_exist_path(path)
    }

    async fn is_exist_in_folder<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        folder_id: i32,
    ) -> Result<bool> {
        self.inner.is_exist_in_folder(folder_id)
    }

    async fn update_path<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        old_path: &LibTrackPath,
        new_path: &LibTrackPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()> {
        self.inner.update_path(old_path, new_path, new_folder_id)
    }

    async fn update_duration<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        track_id: i32,
        duration: u32,
    ) -> Result<()> {
        self.inner.update_duration(track_id, duration)
    }

    async fn delete<'c>(&self, _db: &mut PgTransaction<'c>, track_id: i32) -> Result<()> {
        self.inner.delete(track_id)
    }
}
mock! {
    pub DbTrackRepositoryInner {
        pub fn get_id_by_path(
            &self,
            path: &LibTrackPath,
        ) -> Result<Option<i32>>;

        pub fn get_path_by_path_str(
            &self,
            path: &LibPathStr,
        ) -> Result<Vec<LibTrackPath>>;

        pub fn get_all_path(&self) -> Result<Vec<LibTrackPath>> ;

        pub fn get_path_by_directory(
            &self,
            path: &LibDirPath,
        ) -> Result<Vec<LibTrackPath>>;

        pub fn get_path_all(&self) -> Result<Vec<LibTrackPath>>;

        pub fn path_str_as_track_path(
            &self,
            path_str: &LibPathStr,
        ) -> Result<Option<LibTrackPath>>;

        pub fn is_exist_path(&self, path: &LibTrackPath) -> Result<bool>;

        pub fn is_exist_in_folder(&self, folder_id: i32) -> Result<bool>;

        pub fn update_path(
            &self,
            old_path: &LibTrackPath,
            new_path: &LibTrackPath,
            new_folder_id: FolderIdMayRoot,
        ) -> Result<()>;

        pub fn update_duration(
            &self,
            track_id: i32,
            duration: u32,
        ) -> Result<()>;

        pub fn delete(&self, track_id: i32) -> Result<()>;
    }
}
