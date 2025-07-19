use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use mockall::mock;

use super::{DbFolderRepository, FolderIdMayRoot};
use crate::{Error, db::DbTransaction, path::LibDirPath, song::DbSongRepository};

/// ライブラリのフォルダ関係のUsecase
#[async_trait]
pub trait FolderUsecase {
    /// フォルダに曲が含まれてない場合、削除する
    ///
    /// # Arguments
    /// - folder_id: 確認・削除対象のフォルダID
    async fn delete_db_if_empty<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_path: &LibDirPath,
    ) -> Result<()>;
}

/// FolderUsecaseの本実装
#[derive(new)]
pub struct FolderUsecaseImpl<FR, SR>
where
    FR: DbFolderRepository + Sync + Send,
    SR: DbSongRepository + Sync + Send,
{
    db_folder_repository: FR,
    db_song_repository: SR,
}

#[async_trait]
impl<FR, SR> FolderUsecase for FolderUsecaseImpl<FR, SR>
where
    FR: DbFolderRepository + Sync + Send,
    SR: DbSongRepository + Sync + Send,
{
    /// フォルダに曲が含まれてない場合、削除する
    ///
    /// # Arguments
    /// - folder_path: 確認・削除対象のフォルダパス
    async fn delete_db_if_empty<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_path: &LibDirPath,
    ) -> Result<()> {
        //rootが指定されたら無視
        if folder_path.is_root() {
            return Ok(());
        }

        //IDを取得
        let folder_id = self
            .db_folder_repository
            .get_id_by_path(tx, folder_path)
            .await?
            .ok_or_else(|| Error::DbFolderPathNotFound(folder_path.to_owned()))?;

        self.delete_db_if_empty_by_id(tx, folder_id).await
    }
}

impl<FR, SR> FolderUsecaseImpl<FR, SR>
where
    FR: DbFolderRepository + Sync + Send,
    SR: DbSongRepository + Sync + Send,
{
    /// フォルダに曲が含まれてない場合、削除する(再帰実行用のID指定版)
    ///
    /// # Arguments
    /// - folder_path: 確認・削除対象のフォルダパス
    #[async_recursion]
    async fn delete_db_if_empty_by_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<()> {
        //他の曲が含まれる場合、削除せずに終了
        if self
            .db_song_repository
            .is_exist_in_folder(tx, folder_id)
            .await?
        {
            return Ok(());
        }

        let parent_id_mr = {
            let db_folder_repository = &self.db_folder_repository;
            //他のフォルダが含まれる場合、削除せずに終了
            if db_folder_repository
                .is_exist_in_folder(tx, FolderIdMayRoot::Folder(folder_id))
                .await?
            {
                return Ok(());
            }

            //削除するフォルダ情報を取得
            let parent_id_mr = db_folder_repository
                .get_parent(tx, folder_id)
                .await?
                .ok_or(Error::DbFolderIdNotFound(folder_id))?;

            //削除を実行
            db_folder_repository.delete(tx, folder_id).await?;

            parent_id_mr
        };

        //親フォルダについて再帰実行
        if let FolderIdMayRoot::Folder(parent_id) = parent_id_mr {
            self.delete_db_if_empty_by_id(tx, parent_id).await?;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct MockFolderUsecase {
    pub inner: MockFolderUsecaseInner,
}
#[async_trait]
impl FolderUsecase for MockFolderUsecase {
    async fn delete_db_if_empty<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        folder_path: &LibDirPath,
    ) -> Result<()> {
        self.inner.delete_db_if_empty(folder_path)
    }
}
mock! {
    pub FolderUsecaseInner {
        pub fn delete_db_if_empty(
            &self,
            folder_path: &LibDirPath,
        ) -> Result<()>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{folder::MockDbFolderRepository, mocks, song::MockDbSongRepository};
    use paste::paste;

    mocks! {
        FolderUsecaseImpl,
        [DbFolderRepository, DbSongRepository]
    }

    #[test]
    fn test_delete_db_if_empty_trans_once() {
        let mut mocks = Mocks::new();
        mocks.db_song_repository(|m| {
            m.expect_is_exist_in_folder()
                .withf(|_, a_folder_id| *a_folder_id == 15)
                .times(1)
                .returning(|_, _| Ok(false));

            m.expect_is_exist_in_folder()
                .withf(|_, a_folder_id| *a_folder_id == 4)
                .times(1)
                .returning(|_, _| Ok(true));
        });
        mocks.db_folder_repository(|m| {
            m.expect_is_exist_in_folder()
                .withf(|_, a_folder_id| *a_folder_id == FolderIdMayRoot::Folder(15))
                .times(1)
                .returning(|_, _| Ok(false));
            m.expect_get_parent()
                .withf(|_, a_folder_id| *a_folder_id == 15)
                .times(1)
                .returning(|_, _| Ok(Some(FolderIdMayRoot::Folder(4))));
            m.expect_delete()
                .withf(|_, a_folder_id| *a_folder_id == 15)
                .times(1)
                .returning(|_, _| Ok(()));

            m.expect_delete()
                .withf(|_, a_folder_id| *a_folder_id == 4)
                .times(0);
        });

        let mut tx = DbTransaction::Dummy;

        mocks.run_target(|t| {
            t.delete_db_if_empty_by_id(&mut tx, 15).unwrap();
        });
    }
    #[test]
    fn test_delete_db_if_empty_folder_exists() {
        let mut mocks = Mocks::new();
        mocks.db_song_repository(|m| {
            m.expect_is_exist_in_folder()
                .withf(|_, a_folder_id| *a_folder_id == 15)
                .times(1)
                .returning(|_, _| Ok(false));
        });
        mocks.db_folder_repository(|m| {
            m.expect_is_exist_in_folder()
                .withf(|_, a_folder_id| *a_folder_id == FolderIdMayRoot::Folder(15))
                .times(1)
                .returning(|_, _| Ok(true));
            m.expect_delete().times(0);
        });

        let mut tx = DbTransaction::Dummy;

        mocks.run_target(|t| {
            t.delete_db_if_empty_by_id(&mut tx, 15).unwrap();
        });
    }
    #[test]
    fn test_delete_db_if_empty_trans_root_check() {
        let mut mocks = Mocks::new();
        mocks.db_song_repository(|m| {
            m.expect_is_exist_in_folder()
                .times(1)
                .returning(|_, a_folder_id| {
                    assert_eq!(a_folder_id, 15);
                    Ok(false)
                });
        });
        mocks.db_folder_repository(|m| {
            m.expect_is_exist_in_folder()
                .times(1)
                .returning(|_, a_folder_id| {
                    assert_eq!(a_folder_id, FolderIdMayRoot::Folder(15));
                    Ok(false)
                });
            m.expect_get_parent()
                .withf(|_, a_folder_id| *a_folder_id == 15)
                .times(1)
                .returning(|_, _| Ok(Some(FolderIdMayRoot::Root)));
            m.expect_delete().times(1).returning(|_, a_folder_id| {
                assert_eq!(a_folder_id, 15);
                Ok(())
            });
        });

        let mut tx = DbTransaction::Dummy;

        mocks.run_target(|t| {
            t.delete_db_if_empty_by_id(&mut tx, 15).unwrap();
        });
    }
}
