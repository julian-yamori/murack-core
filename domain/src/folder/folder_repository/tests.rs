use anyhow::Result;
use sqlx::PgPool;

use crate::path::LibraryDirectoryPath;

// register_not_exists 関数のテスト
mod test_register_not_exists {
    use std::str::FromStr;

    use super::*;

    /// 複数階層のフォルダパスを登録するテスト
    /// 途中の親フォルダは存在するが、末端の2階層は存在しない場合
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_register_not_exists_2dir")
    )]
    async fn 存在しない2階層を作成(pool: PgPool) -> Result<()> {
        let lib_dir_path = LibraryDirectoryPath::from_str("test/hoge/fuga")?;

        let mut tx = pool.begin().await?;

        super::super::register_not_exists(&mut tx, &lib_dir_path).await?;

        // 対象フォルダが実際に作成されたことを確認
        let exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM folder_paths WHERE path = $1",
            "test/hoge/fuga/"
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(exists, Some(1));

        // 親フォルダも作成されたことを確認
        let parent_exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM folder_paths WHERE path = $1",
            "test/hoge/"
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(parent_exists, Some(1));

        // 元々存在していた "test" フォルダが残っていることを確認（fixture で INSERT しているもの）
        let root_test_exists =
            sqlx::query_scalar!("SELECT COUNT(*) FROM folder_paths WHERE path = $1", "test/")
                .fetch_one(&mut *tx)
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
        let lib_dir_path = LibraryDirectoryPath::from_str("test")?;

        let mut tx = pool.begin().await?;

        super::super::register_not_exists(&mut tx, &lib_dir_path).await?;

        // 対象フォルダが実際に作成されたことを確認
        let exists =
            sqlx::query_scalar!("SELECT COUNT(*) FROM folder_paths WHERE path = $1", "test/")
                .fetch_one(&mut *tx)
                .await?;
        assert_eq!(exists, Some(1));

        // parent_id は NULL（Root直下）であることを確認
        let parent_id = sqlx::query_scalar!(
            "SELECT parent_id FROM folder_paths WHERE path = $1",
            "test/"
        )
        .fetch_one(&mut *tx)
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
        let lib_dir_path = LibraryDirectoryPath::from_str("test/hoge/fuga")?;

        let mut tx = pool.begin().await?;

        assert_eq!(
            super::super::register_not_exists(&mut tx, &lib_dir_path).await?,
            // 結果は既存のフォルダID (12) を返すはず
            12
        );

        // フォルダ数が変わっていないことを確認（新規作成されていない）
        let total_count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM folder_paths"#)
            .fetch_one(&mut *tx)
            .await?;
        assert_eq!(total_count, 3); // fixture で3個作成している

        Ok(())
    }
}

// delete_if_empty 関数のテスト
mod test_delete_if_empty {
    use super::*;

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/delete_if_empty/delete_empty_with_parent_has_tracks.sql")
    )]
    async fn parent_has_tracks(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        // "music/empty/" フォルダを削除実行
        super::super::delete_if_empty(&mut tx, &"music/empty/".to_string().try_into()?).await?;

        // "music/empty/" フォルダが削除されたことを確認
        let target_id = sqlx::query_scalar!(
            "SELECT id FROM folder_paths WHERE path = $1",
            "music/empty/"
        )
        .fetch_optional(&mut *tx)
        .await?;
        assert!(target_id.is_none());

        // "music/" フォルダは削除されていないことを確認（曲があるため）
        let parent_id =
            sqlx::query_scalar!("SELECT id FROM folder_paths WHERE path = $1", "music/")
                .fetch_optional(&mut *tx)
                .await?;
        assert!(parent_id.is_some());

        // 曲は残っていることを確認
        let track_count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM tracks"#,)
            .fetch_one(&mut *tx)
            .await?;
        assert_eq!(track_count, 2);

        Ok(())
    }

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/delete_if_empty/delete_folder_with_subfolders.sql")
    )]
    async fn with_subfolders(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        // "music/" フォルダを削除実行（サブフォルダがあるので削除されないはず）
        super::super::delete_if_empty(&mut tx, &"music/".to_string().try_into()?).await?;

        // "music/" フォルダが削除されていないことを確認
        let target_id =
            sqlx::query_scalar!("SELECT id FROM folder_paths WHERE path = $1", "music/")
                .fetch_optional(&mut *tx)
                .await?;
        assert!(target_id.is_some());

        // サブフォルダも残っていることを確認
        let child_id = sqlx::query_scalar!(
            "SELECT id FROM folder_paths WHERE path = $1",
            "music/subfolder/"
        )
        .fetch_optional(&mut *tx)
        .await?;
        assert!(child_id.is_some());

        Ok(())
    }

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/delete_if_empty/delete_empty_folder_under_root.sql")
    )]
    async fn delete_under_root(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        // "music/" フォルダを削除実行
        super::super::delete_if_empty(&mut tx, &"music/".to_string().try_into()?).await?;

        // "music/" フォルダが削除されたことを確認
        let target_id =
            sqlx::query_scalar!("SELECT id FROM folder_paths WHERE path = $1", "music/")
                .fetch_optional(&mut *tx)
                .await?;
        assert!(target_id.is_none());

        // 無関係なフォルダは残っていることを確認
        let remain_id = sqlx::query_scalar!(
            "SELECT id FROM folder_paths WHERE path = $1",
            "otherfolder/"
        )
        .fetch_optional(&mut *tx)
        .await?;
        assert!(remain_id.is_some());

        Ok(())
    }
}
