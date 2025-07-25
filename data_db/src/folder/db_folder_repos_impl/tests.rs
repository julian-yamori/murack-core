use anyhow::Result;
use murack_core_domain::{
    db::DbTransaction,
    folder::{DbFolderRepository, FolderIdMayRoot},
    path::LibDirPath,
};
use sqlx::PgPool;

use crate::folder::DbFolderRepositoryImpl;

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

        let target = DbFolderRepositoryImpl::new();

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

    /// ルート直下にフォルダを作成するテスト
    /// データベースが空の状態から開始
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_register_not_exists_root")
    )]
    async fn ルート直下に作成(pool: PgPool) -> Result<()> {
        let lib_dir_path = LibDirPath::new("test");

        let target = DbFolderRepositoryImpl::new();

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target.register_not_exists(&mut tx, &lib_dir_path).await?;

        // 結果は Root 以外の新しいフォルダのIDを返すはず
        assert!(matches!(result, FolderIdMayRoot::Folder(_)));

        // 対象フォルダが実際に作成されたことを確認
        let exists =
            sqlx::query_scalar!("SELECT COUNT(*) FROM folder_paths WHERE path = $1", "test/")
                .fetch_one(&mut **tx.get())
                .await?;
        assert_eq!(exists, Some(1));

        // parent_id は NULL（Root直下）であることを確認
        let parent_id = sqlx::query_scalar!(
            "SELECT parent_id FROM folder_paths WHERE path = $1",
            "test/"
        )
        .fetch_one(&mut **tx.get())
        .await?;
        assert_eq!(parent_id, None);

        Ok(())
    }

    /// 既に存在するフォルダを指定した場合のテスト
    /// 既存のIDを返し、新規作成は行わない
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_register_not_exists_exists")
    )]
    async fn 既に存在する場合(pool: PgPool) -> Result<()> {
        let lib_dir_path = LibDirPath::new("test/hoge/fuga");

        let target = DbFolderRepositoryImpl::new();

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target.register_not_exists(&mut tx, &lib_dir_path).await?;

        // 結果は既存のフォルダID (12) を返すはず
        assert_eq!(result, FolderIdMayRoot::Folder(12));

        // フォルダ数が変わっていないことを確認（新規作成されていない）
        let total_count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM folder_paths"#)
            .fetch_one(&mut **tx.get())
            .await?;
        assert_eq!(total_count, 3); // fixture で3個作成している

        Ok(())
    }
}
