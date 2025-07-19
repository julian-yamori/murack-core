use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use crate::{db::DbTransaction, folder::FolderIdMayRoot, path::LibDirPath};

/// フォルダ関係のDBリポジトリ
#[async_trait]
pub trait DbFolderRepository {
    /// 指定されたフォルダのIDを取得
    async fn get_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>>;

    /// 指定されたフォルダの、親フォルダのIDを取得
    async fn get_parent<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderIdMayRoot>>;

    /// 指定されたパスのフォルダが存在するか確認
    async fn is_exist_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<bool>;

    /// 指定されたフォルダに、子フォルダが存在するか確認
    ///
    /// folder_idにRootを指定した場合、
    /// ルート直下に子フォルダがあるかを調べる
    async fn is_exist_in_folder<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<bool>;

    /// フォルダのパス情報を登録
    ///
    /// 既に同じパスが存在する場合は新規登録せず、IDを返す。
    /// pathがrootの場合も登録せず、FolderIdMayRoot::Rootを返す。
    ///
    /// # Arguments
    /// - path: 登録する、ライブラリフォルダ内のパス
    /// # Return
    /// 新規登録されたデータ、もしくは既存のデータのID。
    async fn register_not_exists<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<FolderIdMayRoot>;

    /// フォルダを削除
    ///
    /// # Arguments
    /// - folder_id: 削除対象のフォルダID
    async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, folder_id: i32) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbFolderRepository {
    pub inner: MockDbFolderRepositoryInner,
}
#[async_trait]
impl DbFolderRepository for MockDbFolderRepository {
    async fn get_id_by_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>> {
        self.inner.get_id_by_path(path)
    }

    async fn get_parent<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderIdMayRoot>> {
        self.inner.get_parent(folder_id)
    }

    async fn is_exist_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<bool> {
        self.inner.is_exist_path(path)
    }

    async fn is_exist_in_folder<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<bool> {
        self.inner.is_exist_in_folder(folder_id)
    }

    async fn register_not_exists<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<FolderIdMayRoot> {
        self.inner.register_not_exists(path)
    }

    async fn delete<'c>(&self, _db: &mut DbTransaction<'c>, folder_id: i32) -> Result<()> {
        self.inner.delete(folder_id)
    }
}

mock! {
    pub DbFolderRepositoryInner {
        pub fn get_id_by_path(
            &self,
            path: &LibDirPath,
        ) -> Result<Option<i32>>;

        pub fn get_parent(
            &self,
            folder_id: i32,
        ) -> Result<Option<FolderIdMayRoot>>;

        pub fn is_exist_path(&self, path: &LibDirPath) -> Result<bool>;

        pub fn is_exist_in_folder(
            &self,
            folder_id: FolderIdMayRoot,
        ) -> Result<bool>;

        pub fn register_not_exists(
            &self,
            path: &LibDirPath,
        ) -> Result<FolderIdMayRoot>;

        pub fn delete(&self, folder_id: i32) -> Result<()>;
    }
}
