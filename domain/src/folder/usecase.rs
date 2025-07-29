#[cfg(test)]
mod tests;

use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use mockall::mock;
use sqlx::PgTransaction;

use super::FolderIdMayRoot;
use crate::{
    Error, folder::folder_repository, path::LibraryDirectoryPath, track::track_repository,
};

/// ライブラリのフォルダ関係のUsecase
#[async_trait]
pub trait FolderUsecase {
    /// フォルダに曲が含まれてない場合、削除する
    ///
    /// # Arguments
    /// - folder_id: 確認・削除対象のフォルダID
    async fn delete_db_if_empty<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        folder_path: &LibraryDirectoryPath,
    ) -> Result<()>;
}

/// FolderUsecaseの本実装
#[derive(new)]
pub struct FolderUsecaseImpl {}

#[async_trait]
impl FolderUsecase for FolderUsecaseImpl {
    /// フォルダに曲が含まれてない場合、削除する
    ///
    /// # Arguments
    /// - folder_path: 確認・削除対象のフォルダパス
    async fn delete_db_if_empty<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        folder_path: &LibraryDirectoryPath,
    ) -> Result<()> {
        //IDを取得
        let folder_id = folder_repository::get_id_by_path(tx, folder_path)
            .await?
            .ok_or_else(|| Error::DbFolderPathNotFound(folder_path.to_owned()))?;

        delete_db_if_empty_by_id(tx, folder_id).await
    }
}

/// フォルダに曲が含まれてない場合、削除する(再帰実行用のID指定版)
///
/// # Arguments
/// - folder_path: 確認・削除対象のフォルダパス
#[async_recursion]
async fn delete_db_if_empty_by_id<'c>(tx: &mut PgTransaction<'c>, folder_id: i32) -> Result<()> {
    //他の曲が含まれる場合、削除せずに終了
    if track_repository::is_exist_in_folder(tx, folder_id).await? {
        return Ok(());
    }

    let parent_id_mr = {
        //他のフォルダが含まれる場合、削除せずに終了
        if folder_repository::is_exist_in_folder(tx, FolderIdMayRoot::Folder(folder_id)).await? {
            return Ok(());
        }

        //削除するフォルダ情報を取得
        let parent_id_mr = folder_repository::get_parent(tx, folder_id)
            .await?
            .ok_or(Error::DbFolderIdNotFound(folder_id))?;

        //削除を実行
        folder_repository::delete(tx, folder_id).await?;

        parent_id_mr
    };

    //親フォルダについて再帰実行
    if let FolderIdMayRoot::Folder(parent_id) = parent_id_mr {
        delete_db_if_empty_by_id(tx, parent_id).await?;
    }

    Ok(())
}
#[derive(Default)]
pub struct MockFolderUsecase {
    pub inner: MockFolderUsecaseInner,
}
#[async_trait]
impl FolderUsecase for MockFolderUsecase {
    async fn delete_db_if_empty<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        folder_path: &LibraryDirectoryPath,
    ) -> Result<()> {
        self.inner.delete_db_if_empty(folder_path)
    }
}
mock! {
    pub FolderUsecaseInner {
        pub fn delete_db_if_empty(
            &self,
            folder_path: &LibraryDirectoryPath,
        ) -> Result<()>;
    }
}
