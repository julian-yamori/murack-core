use anyhow::Result;
use murack_core_domain::{
    db::DbTransaction,
    folder::{DbFolderRepository, FolderIdMayRoot},
    path::LibDirPath,
};
use sqlx::PgPool;

use crate::folder::{DbFolderRepositoryImpl, FolderPathDaoImpl};

#[sqlx::test(
    migrator = "crate::MIGRATOR",
    fixtures("test_register_not_exists_2dir")
)]
async fn test_register_not_exists_2dir(pool: PgPool) -> Result<()> {
    let lib_dir_path = LibDirPath::new("test/hoge/fuga");

    let folder_path_dao = FolderPathDaoImpl {};
    let target = DbFolderRepositoryImpl::new(folder_path_dao);

    let mut tx = DbTransaction::PgTransaction {
        tx: pool.begin().await?,
    };

    let result = target.register_not_exists(&mut tx, &lib_dir_path).await?;

    // The result should be a new folder with some ID (let's verify it's not Root)
    assert!(matches!(result, FolderIdMayRoot::Folder(_)));

    // Verify the folder was actually created by checking if it exists
    let exists = target.is_exist_path(&mut tx, &lib_dir_path).await?;
    assert!(exists);

    // Verify parent folders were also created
    let parent_path = LibDirPath::new("test/hoge");
    let parent_exists = target.is_exist_path(&mut tx, &parent_path).await?;
    assert!(parent_exists);

    // Verify the original "test" folder still exists
    let root_test_path = LibDirPath::new("test");
    let root_test_exists = target.is_exist_path(&mut tx, &root_test_path).await?;
    assert!(root_test_exists);

    Ok(())
}

mod legacy_tests {
    use crate::folder::MockFolderPathDao;

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
