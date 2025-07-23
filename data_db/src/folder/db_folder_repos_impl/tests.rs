use anyhow::Result;
use murack_core_domain::{
    db::DbTransaction,
    folder::{DbFolderRepository, FolderIdMayRoot},
    path::LibDirPath,
};
use sqlx::PgPool;

use crate::folder::{DbFolderRepositoryImpl, FolderPathDaoImpl};

// register_not_exists 関数のテスト
mod test_register_not_exists {
    use super::*;

    /// 複数階層のフォルダパスを登録するテスト
    /// 途中の親フォルダは存在するが、末端の2階層は存在しない場合
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_register_not_exists_2dir")
    )]
    async fn 存在しない2階層を作成(pool: PgPool) -> Result<()> {
        let lib_dir_path = LibDirPath::new("test/hoge/fuga");

        let folder_path_dao = FolderPathDaoImpl {};
        let target = DbFolderRepositoryImpl::new(folder_path_dao);

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target.register_not_exists(&mut tx, &lib_dir_path).await?;

        // 結果は Root 以外の新しいフォルダのIDを返すはず
        assert!(matches!(result, FolderIdMayRoot::Folder(_)));

        // 対象フォルダが実際に作成されたことを確認
        let exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM folder_paths WHERE path = $1",
            "test/hoge/fuga/"
        )
        .fetch_one(&mut **tx.get())
        .await?;
        assert_eq!(exists, Some(1));

        // 親フォルダも作成されたことを確認
        let parent_exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM folder_paths WHERE path = $1",
            "test/hoge/"
        )
        .fetch_one(&mut **tx.get())
        .await?;
        assert_eq!(parent_exists, Some(1));

        // 元々存在していた "test" フォルダが残っていることを確認（fixture で INSERT しているもの）
        let root_test_exists =
            sqlx::query_scalar!("SELECT COUNT(*) FROM folder_paths WHERE path = $1", "test/")
                .fetch_one(&mut **tx.get())
                .await?;
        assert_eq!(root_test_exists, Some(1));

        Ok(())
    }

    /// Legacy tests (Mock ベース) - 段階的に SQLx テストに移行予定
    mod legacy {
        use super::*;
        use crate::folder::MockFolderPathDao;

        fn target() -> DbFolderRepositoryImpl<MockFolderPathDao> {
            DbFolderRepositoryImpl {
                folder_path_dao: MockFolderPathDao::default(),
            }
        }
        fn checkpoint_all(target: &mut DbFolderRepositoryImpl<MockFolderPathDao>) {
            target.folder_path_dao.inner.checkpoint();
        }

        /// ルート直下にフォルダを作成するテスト
        #[tokio::test]
        async fn ルート直下に作成() -> anyhow::Result<()> {
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

        /// 既に存在するフォルダを指定した場合のテスト
        #[tokio::test]
        async fn 既に存在する場合() -> anyhow::Result<()> {
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
}
