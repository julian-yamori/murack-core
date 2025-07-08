use super::FolderPathDao;
use anyhow::Result;
use domain::{
    db_wrapper::TransactionWrapper,
    folder::{DbFolderRepository, FolderIdMayRoot},
    path::LibDirPath,
};
use std::rc::Rc;

/// DbFolderRepositoryの本実装
#[derive(new)]
pub struct DbFolderRepositoryImpl {
    folder_path_dao: Rc<dyn FolderPathDao>,
}

impl DbFolderRepository for DbFolderRepositoryImpl {
    /// 指定されたフォルダのIDを取得
    fn get_id_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>> {
        self.folder_path_dao.select_id_by_path(tx, path)
    }

    /// 指定されたフォルダの、親フォルダのIDを取得
    fn get_parent(
        &self,
        tx: &TransactionWrapper,
        folder_id: i32,
    ) -> Result<Option<FolderIdMayRoot>> {
        Ok(self
            .folder_path_dao
            .select_by_id(tx, folder_id)?
            .map(|f| f.parent_id.into()))
    }

    /// 指定されたパスのフォルダが存在するか確認
    fn is_exist_path<'c>(&self, tx: &TransactionWrapper<'c>, path: &LibDirPath) -> Result<bool> {
        self.folder_path_dao.exists_path(tx, path)
    }

    /// 指定されたフォルダに、子フォルダが存在するか確認
    ///
    /// folder_idにRootを指定した場合、
    /// ルート直下に子フォルダがあるかを調べる
    fn is_exist_in_folder(
        &self,
        tx: &TransactionWrapper,
        folder_id: FolderIdMayRoot,
    ) -> Result<bool> {
        let folder_count = self.folder_path_dao.count_by_parent_id(tx, folder_id)?;
        Ok(folder_count > 0)
    }

    /// フォルダのパス情報を登録
    ///
    /// 既に同じパスが存在する場合は新規登録せず、IDを返す
    ///
    /// # Arguments
    /// - path: 登録する、ライブラリフォルダ内のパス
    /// # Return
    /// 新規登録されたデータ、もしくは既存のデータのID。
    fn register_not_exists(
        &self,
        tx: &TransactionWrapper,
        path: &LibDirPath,
    ) -> Result<FolderIdMayRoot> {
        //ライブラリルートならNone
        if path.is_root() {
            return Ok(FolderIdMayRoot::Root);
        }

        //同一パスのデータを検索し、そのIDを取得
        let existing_id = self.folder_path_dao.select_id_by_path(tx, path)?;

        //見つかった場合はこのIDを返す
        if let Some(i) = existing_id {
            return Ok(FolderIdMayRoot::Folder(i));
        }

        //親ディレクトリについて再帰呼出し、親のID取得
        let parent_id = self.register_not_exists(tx, &path.parent().unwrap())?;

        let my_name = path.dir_name().unwrap();

        let new_id = self.folder_path_dao.insert(tx, path, my_name, parent_id)?;

        Ok(FolderIdMayRoot::Folder(new_id))
    }

    /// フォルダを削除
    ///
    /// # Arguments
    /// - folder_id: 削除対象のフォルダID
    fn delete(&self, tx: &TransactionWrapper, folder_id: i32) -> Result<()> {
        self.folder_path_dao.delete_by_id(tx, folder_id)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::MockFolderPathDao;
    use super::*;
    use domain::{db_wrapper::ConnectionFactory, mocks};
    use paste::paste;

    mocks! {
        DbFolderRepositoryImpl,
        [FolderPathDao]
    }

    #[test]
    fn test_register_not_exists_2dir() {
        fn lib_dir_path() -> LibDirPath {
            LibDirPath::new("test/hoge/fuga")
        }
        fn lib_dir_path_p1() -> LibDirPath {
            LibDirPath::new("test/hoge")
        }
        fn lib_dir_path_p2() -> LibDirPath {
            LibDirPath::new("test")
        }

        let mut mocks = Mocks::new();
        mocks.folder_path_dao(|m| {
            m.expect_select_id_by_path()
                .withf(|_, a_path| a_path == &lib_dir_path())
                .returning(|_, _| Ok(None));
            m.expect_select_id_by_path()
                .withf(|_, a_path| a_path == &lib_dir_path_p1())
                .returning(|_, _| Ok(None));
            m.expect_select_id_by_path()
                .withf(|_, a_path| a_path == &lib_dir_path_p2())
                .returning(|_, _| Ok(Some(2)));

            m.expect_insert()
                .withf(|_, a_path, _, _| a_path == &lib_dir_path())
                .times(1)
                .returning(|_, _, a_name, a_parent_id| {
                    assert_eq!(a_name, "fuga");
                    assert_eq!(a_parent_id, FolderIdMayRoot::Folder(4));
                    Ok(5)
                });
            m.expect_insert()
                .withf(|_, a_path, _, _| a_path == &lib_dir_path_p1())
                .times(1)
                .returning(|_, _, a_name, a_parent_id| {
                    assert_eq!(a_parent_id, FolderIdMayRoot::Folder(2));
                    assert_eq!(a_name, "hoge");
                    Ok(4)
                });
            m.expect_insert()
                .withf(|_, a_path, _, _| a_path == &lib_dir_path_p2())
                .times(0);
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let result = t.register_not_exists(&tx, &lib_dir_path()).unwrap();
            assert_eq!(result, FolderIdMayRoot::Folder(5));
        });
    }
    #[test]
    fn test_register_not_exists_register_root() {
        fn lib_dir_path() -> LibDirPath {
            LibDirPath::new("test")
        }

        let mut mocks = Mocks::new();
        mocks.folder_path_dao(|m| {
            m.expect_select_id_by_path()
                .withf(|_, a_path| a_path == &lib_dir_path())
                .returning(|_, _| Ok(None));
            m.expect_select_id_by_path()
                .withf(|_, a_path| a_path == &LibDirPath::new(""))
                .times(0);

            m.expect_insert()
                .withf(|_, a_path, _, _| a_path == &lib_dir_path())
                .times(1)
                .returning(|_, _, a_name, a_parent_id| {
                    assert_eq!(a_name, "test");
                    assert_eq!(a_parent_id, FolderIdMayRoot::Root);
                    Ok(99)
                });
            m.expect_insert()
                .withf(|_, a_path, _, _| a_path == &LibDirPath::new(""))
                .times(0);
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let result = t.register_not_exists(&tx, &lib_dir_path()).unwrap();
            assert_eq!(result, FolderIdMayRoot::Folder(99));
        });
    }
    #[test]
    fn test_register_not_exists_exists() {
        fn lib_dir_path() -> LibDirPath {
            LibDirPath::new("test/hoge/fuga")
        }

        let mut mocks = Mocks::new();
        mocks.folder_path_dao(|m| {
            m.expect_select_id_by_path()
                .withf(|_, a_path| a_path == &lib_dir_path())
                .returning(|_, _| Ok(Some(12)));
            m.expect_insert().times(0);
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let result = t.register_not_exists(&tx, &lib_dir_path()).unwrap();
            assert_eq!(result, FolderIdMayRoot::Folder(12));
        });
    }
}
