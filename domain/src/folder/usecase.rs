use super::{DbFolderRepository, FolderIdMayRoot};
use crate::{db_wrapper::TransactionWrapper, path::LibDirPath, song::DbSongRepository, Error};
use anyhow::Result;
use mockall::automock;
use std::rc::Rc;

/// ライブラリのフォルダ関係のUsecase
#[automock]
pub trait FolderUsecase {
    /// フォルダに曲が含まれてない場合、削除する
    ///
    /// # Arguments
    /// - folder_id: 確認・削除対象のフォルダID
    fn delete_db_if_empty<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        folder_path: &LibDirPath,
    ) -> Result<()>;
}

/// FolderUsecaseの本実装
#[derive(new)]
pub struct FolderUsecaseImpl {
    db_folder_repository: Rc<dyn DbFolderRepository>,
    db_song_repository: Rc<dyn DbSongRepository>,
}

impl FolderUsecase for FolderUsecaseImpl {
    /// フォルダに曲が含まれてない場合、削除する
    ///
    /// # Arguments
    /// - folder_path: 確認・削除対象のフォルダパス
    fn delete_db_if_empty<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        folder_path: &LibDirPath,
    ) -> Result<()> {
        //rootが指定されたら無視
        if folder_path.is_root() {
            return Ok(());
        }

        //IDを取得
        let folder_id = self
            .db_folder_repository
            .get_id_by_path(tx, folder_path)?
            .ok_or_else(|| Error::DbFolderPathNotFound(folder_path.to_owned()))?;

        self.delete_db_if_empty_by_id(tx, folder_id)
    }
}

impl FolderUsecaseImpl {
    /// フォルダに曲が含まれてない場合、削除する(再帰実行用のID指定版)
    ///
    /// # Arguments
    /// - folder_path: 確認・削除対象のフォルダパス
    fn delete_db_if_empty_by_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        folder_id: i32,
    ) -> Result<()> {
        //他の曲が含まれる場合、削除せずに終了
        if self.db_song_repository.is_exist_in_folder(tx, folder_id)? {
            return Ok(());
        }

        let parent_id_mr = {
            let db_folder_repository = &self.db_folder_repository;
            //他のフォルダが含まれる場合、削除せずに終了
            if db_folder_repository.is_exist_in_folder(tx, FolderIdMayRoot::Folder(folder_id))? {
                return Ok(());
            }

            //削除するフォルダ情報を取得
            let parent_id_mr = db_folder_repository
                .get_parent(tx, folder_id)?
                .ok_or(Error::DbFolderIdNotFound(folder_id))?;

            //削除を実行
            db_folder_repository.delete(tx, folder_id)?;

            parent_id_mr
        };

        //親フォルダについて再帰実行
        if let FolderIdMayRoot::Folder(parent_id) = parent_id_mr {
            self.delete_db_if_empty_by_id(tx, parent_id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db_wrapper::ConnectionFactory, folder::MockDbFolderRepository, mocks,
        song::MockDbSongRepository,
    };
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

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            t.delete_db_if_empty_by_id(&tx, 15).unwrap();
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

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            t.delete_db_if_empty_by_id(&tx, 15).unwrap();
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

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            t.delete_db_if_empty_by_id(&tx, 15).unwrap();
        });
    }
}
