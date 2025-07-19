use anyhow::Result;
use async_trait::async_trait;
use domain::{db::DbTransaction, folder::FolderIdMayRoot, path::LibDirPath};
use mockall::mock;

use super::FolderPathRow;
use crate::converts::{DbOptionString, enums::db_from_folder_id_may_root};

/// folder_pathテーブルのDAO
#[async_trait]
pub trait FolderPathDao {
    /// IDを指定して検索
    async fn select_by_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderPathRow>>;

    /// パスを指定して検索
    async fn select_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<FolderPathRow>>;

    /// パスを指定し、IDを取得
    async fn select_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>>;

    /// 全レコード数を取得
    async fn count_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<u32>;

    /// 親フォルダIDを指定してレコード数を取得
    async fn count_by_parent_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        parent_id: FolderIdMayRoot,
    ) -> Result<u32>;

    /// 指定されたpathのレコードが存在するか確認
    async fn exists_path<'c>(&self, tx: &mut DbTransaction<'c>, path: &LibDirPath) -> Result<bool>;

    /// 新規登録
    ///
    /// # Return
    /// 登録されたレコードのrowid
    async fn insert<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
        name: &str,
        parent_id: FolderIdMayRoot,
    ) -> Result<i32>;

    /// IDを指定してフォルダを削除
    async fn delete_by_id<'c>(&self, tx: &mut DbTransaction<'c>, folder_id: i32) -> Result<()>;
}

/// FolderPathDaoの本実装
pub struct FolderPathDaoImpl {}

#[async_trait]
impl FolderPathDao for FolderPathDaoImpl {
    /// IDを指定して検索
    async fn select_by_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderPathRow>> {
        let row = sqlx::query_as!(
            FolderPathRow,
            r#"SELECT id, path, name AS "name: DbOptionString", parent_id FROM folder_paths WHERE id = $1"#,
            folder_id
        )
        .fetch_optional(&mut **tx.get()).await?;

        Ok(row)
    }

    /// パスを指定して検索
    async fn select_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<FolderPathRow>> {
        let row = sqlx::query_as!(
            FolderPathRow,
            "SELECT id, path, name, parent_id FROM folder_paths WHERE path = $1",
            path.as_str()
        )
        .fetch_optional(&mut **tx.get())
        .await?;

        Ok(row)
    }

    /// パスを指定し、IDを取得
    async fn select_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>> {
        let id = sqlx::query_scalar!("SELECT id FROM folder_paths WHERE path = $1", path.as_str())
            .fetch_optional(&mut **tx.get())
            .await?;

        Ok(id)
    }

    /// 全レコード数を取得
    async fn count_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<u32> {
        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM folder_paths"#)
            .fetch_one(&mut **tx.get())
            .await?;

        Ok(count.try_into()?)
    }

    /// 親フォルダIDを指定してレコード数を取得
    async fn count_by_parent_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        parent_id: FolderIdMayRoot,
    ) -> Result<u32> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE parent_id IS NOT DISTINCT FROM $1"#,
            db_from_folder_id_may_root(parent_id),
        )
        .fetch_one(&mut **tx.get()).await?;

        Ok(count.try_into()?)
    }

    /// 指定されたpathのレコードが存在するか確認
    async fn exists_path<'c>(&self, tx: &mut DbTransaction<'c>, path: &LibDirPath) -> Result<bool> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE path = $1"#,
            path.as_str(),
        )
        .fetch_one(&mut **tx.get())
        .await?;

        Ok(count > 0)
    }

    /// 新規登録
    ///
    /// # Return
    /// 登録されたレコードのrowid
    async fn insert<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
        name: &str,
        parent_id: FolderIdMayRoot,
    ) -> Result<i32> {
        let id = sqlx::query_scalar!(
            "INSERT INTO folder_paths (path, name, parent_id) VALUES($1, $2, $3) RETURNING id",
            path.as_str(),
            name,
            db_from_folder_id_may_root(parent_id)
        )
        .fetch_one(&mut **tx.get())
        .await?;

        Ok(id)
    }

    /// IDを指定してフォルダを削除
    async fn delete_by_id<'c>(&self, tx: &mut DbTransaction<'c>, folder_id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM folder_paths WHERE id = $1", folder_id,)
            .execute(&mut **tx.get())
            .await?;

        Ok(())
    }
}

#[derive(Default)]
pub struct MockFolderPathDao {
    pub inner: MockFolderPathDaoInner,
}
#[async_trait]
impl FolderPathDao for MockFolderPathDao {
    async fn select_by_id<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderPathRow>> {
        self.inner.select_by_id(folder_id)
    }

    async fn select_by_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<FolderPathRow>> {
        self.inner.select_by_path(path)
    }

    async fn select_id_by_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>> {
        self.inner.select_id_by_path(path)
    }

    async fn count_all<'c>(&self, _db: &mut DbTransaction<'c>) -> Result<u32> {
        self.inner.count_all()
    }

    async fn count_by_parent_id<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        parent_id: FolderIdMayRoot,
    ) -> Result<u32> {
        self.inner.count_by_parent_id(parent_id)
    }

    async fn exists_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<bool> {
        self.inner.exists_path(path)
    }

    async fn insert<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibDirPath,
        name: &str,
        parent_id: FolderIdMayRoot,
    ) -> Result<i32> {
        self.inner.insert(path, name, parent_id)
    }

    async fn delete_by_id<'c>(&self, _db: &mut DbTransaction<'c>, folder_id: i32) -> Result<()> {
        self.inner.delete_by_id(folder_id)
    }
}
mock! {
    pub FolderPathDaoInner {
        pub fn select_by_id(
            &self,
            folder_id: i32,
        ) -> Result<Option<FolderPathRow>>;

        pub fn select_by_path(
            &self,
            path: &LibDirPath,
        ) -> Result<Option<FolderPathRow>>;

        pub fn select_id_by_path(
            &self,
            path: &LibDirPath,
        ) -> Result<Option<i32>>;

        pub fn count_all(&self) -> Result<u32>;

        pub fn count_by_parent_id(
            &self,
            parent_id: FolderIdMayRoot,
        ) -> Result<u32>;

        pub fn exists_path(&self, path: &LibDirPath) -> Result<bool>;

        pub fn insert(
            &self,
            path: &LibDirPath,
            name: &str,
            parent_id: FolderIdMayRoot,
        ) -> Result<i32>;

        pub fn delete_by_id(&self, folder_id: i32) -> Result<()>;
    }
}
