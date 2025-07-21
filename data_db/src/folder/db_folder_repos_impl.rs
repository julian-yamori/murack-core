use anyhow::Result;
use async_trait::async_trait;
use domain::{
    db::DbTransaction,
    folder::{DbFolderRepository, FolderIdMayRoot},
    path::LibDirPath,
};

use crate::converts::enums::db_into_folder_id_may_root;

use super::FolderPathDao;

/// DbFolderRepositoryの本実装
#[derive(new)]
pub struct DbFolderRepositoryImpl<FPD>
where
    FPD: FolderPathDao + Sync + Send,
{
    folder_path_dao: FPD,
}

#[async_trait]
impl<FPD> DbFolderRepository for DbFolderRepositoryImpl<FPD>
where
    FPD: FolderPathDao + Sync + Send,
{
    /// 指定されたフォルダのIDを取得
    async fn get_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>> {
        self.folder_path_dao.select_id_by_path(tx, path).await
    }

    /// 指定されたフォルダの、親フォルダのIDを取得
    async fn get_parent<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderIdMayRoot>> {
        Ok(self
            .folder_path_dao
            .select_by_id(tx, folder_id)
            .await?
            .map(|f| db_into_folder_id_may_root(f.parent_id)))
    }

    /// 指定されたパスのフォルダが存在するか確認
    async fn is_exist_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<bool> {
        self.folder_path_dao.exists_path(tx, path).await
    }

    /// 指定されたフォルダに、子フォルダが存在するか確認
    ///
    /// folder_idにRootを指定した場合、
    /// ルート直下に子フォルダがあるかを調べる
    async fn is_exist_in_folder<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<bool> {
        let folder_count = self
            .folder_path_dao
            .count_by_parent_id(tx, folder_id)
            .await?;
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
    async fn register_not_exists<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<FolderIdMayRoot> {
        //ライブラリルートならNone
        if path.is_root() {
            return Ok(FolderIdMayRoot::Root);
        }

        //同一パスのデータを検索し、そのIDを取得
        let existing_id = self.folder_path_dao.select_id_by_path(tx, path).await?;

        //見つかった場合はこのIDを返す
        if let Some(i) = existing_id {
            return Ok(FolderIdMayRoot::Folder(i));
        }

        //親ディレクトリについて再帰呼出し、親のID取得
        let parent_id = self
            .register_not_exists(tx, &path.parent().unwrap())
            .await?;

        let my_name = path.dir_name().unwrap();

        let new_id = self
            .folder_path_dao
            .insert(tx, path, my_name, parent_id)
            .await?;

        Ok(FolderIdMayRoot::Folder(new_id))
    }

    /// フォルダを削除
    ///
    /// # Arguments
    /// - folder_id: 削除対象のフォルダID
    async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, folder_id: i32) -> Result<()> {
        self.folder_path_dao.delete_by_id(tx, folder_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::MockFolderPathDao;
    use super::*;

    fn target() -> DbFolderRepositoryImpl<MockFolderPathDao> {
        DbFolderRepositoryImpl {
            folder_path_dao: MockFolderPathDao::default(),
        }
    }
    fn checkpoint_all(target: &mut DbFolderRepositoryImpl<MockFolderPathDao>) {
        target.folder_path_dao.inner.checkpoint();
    }

    #[tokio::test]
    async fn test_register_not_exists_2dir() -> anyhow::Result<()> {
        fn lib_dir_path() -> LibDirPath {
            LibDirPath::new("test/hoge/fuga")
        }
        fn lib_dir_path_p1() -> LibDirPath {
            LibDirPath::new("test/hoge")
        }
        fn lib_dir_path_p2() -> LibDirPath {
            LibDirPath::new("test")
        }

        let mut target = target();
        target
            .folder_path_dao
            .inner
            .expect_select_id_by_path()
            .withf(|a_path| a_path == &lib_dir_path())
            .returning(|_| Ok(None));
        target
            .folder_path_dao
            .inner
            .expect_select_id_by_path()
            .withf(|a_path| a_path == &lib_dir_path_p1())
            .returning(|_| Ok(None));
        target
            .folder_path_dao
            .inner
            .expect_select_id_by_path()
            .withf(|a_path| a_path == &lib_dir_path_p2())
            .returning(|_| Ok(Some(2)));
        target
            .folder_path_dao
            .inner
            .expect_insert()
            .withf(|a_path, _, _| a_path == &lib_dir_path())
            .times(1)
            .returning(|_, a_name, a_parent_id| {
                assert_eq!(a_name, "fuga");
                assert_eq!(a_parent_id, FolderIdMayRoot::Folder(4));
                Ok(5)
            });
        target
            .folder_path_dao
            .inner
            .expect_insert()
            .withf(|a_path, _, _| a_path == &lib_dir_path_p1())
            .times(1)
            .returning(|_, a_name, a_parent_id| {
                assert_eq!(a_parent_id, FolderIdMayRoot::Folder(2));
                assert_eq!(a_name, "hoge");
                Ok(4)
            });
        target
            .folder_path_dao
            .inner
            .expect_insert()
            .withf(|a_path, _, _| a_path == &lib_dir_path_p2())
            .times(0);

        let mut tx = DbTransaction::Dummy;

        let result = target.register_not_exists(&mut tx, &lib_dir_path()).await?;
        assert_eq!(result, FolderIdMayRoot::Folder(5));

        checkpoint_all(&mut target);
        Ok(())
    }

    #[tokio::test]
    async fn test_register_not_exists_register_root() -> anyhow::Result<()> {
        fn lib_dir_path() -> LibDirPath {
            LibDirPath::new("test")
        }

        let mut target = target();
        target
            .folder_path_dao
            .inner
            .expect_select_id_by_path()
            .withf(|a_path| a_path == &lib_dir_path())
            .returning(|_| Ok(None));
        target
            .folder_path_dao
            .inner
            .expect_select_id_by_path()
            .withf(|a_path| a_path == &LibDirPath::new(""))
            .times(0);
        target
            .folder_path_dao
            .inner
            .expect_insert()
            .withf(|a_path, _, _| a_path == &lib_dir_path())
            .times(1)
            .returning(|_, a_name, a_parent_id| {
                assert_eq!(a_name, "test");
                assert_eq!(a_parent_id, FolderIdMayRoot::Root);
                Ok(99)
            });
        target
            .folder_path_dao
            .inner
            .expect_insert()
            .withf(|a_path, _, _| a_path == &LibDirPath::new(""))
            .times(0);

        let mut tx = DbTransaction::Dummy;

        let result = target.register_not_exists(&mut tx, &lib_dir_path()).await?;
        assert_eq!(result, FolderIdMayRoot::Folder(99));

        checkpoint_all(&mut target);
        Ok(())
    }

    #[tokio::test]
    async fn test_register_not_exists_exists() -> anyhow::Result<()> {
        fn lib_dir_path() -> LibDirPath {
            LibDirPath::new("test/hoge/fuga")
        }

        let mut target = target();
        target
            .folder_path_dao
            .inner
            .expect_select_id_by_path()
            .withf(|a_path| a_path == &lib_dir_path())
            .returning(|_| Ok(Some(12)));
        target.folder_path_dao.inner.expect_insert().times(0);

        let mut tx = DbTransaction::Dummy;

        let result = target.register_not_exists(&mut tx, &lib_dir_path()).await?;
        assert_eq!(result, FolderIdMayRoot::Folder(12));

        checkpoint_all(&mut target);
        Ok(())
    }
}
