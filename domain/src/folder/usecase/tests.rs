use anyhow::Result;
use sqlx::PgPool;

// delete_db_if_empty 関数のテスト
mod delete_db_if_empty {
    use super::*;

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("delete_empty_with_parent_has_tracks")
    )]
    async fn parent_has_tracks(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        // "music/empty/" フォルダを削除実行
        super::super::delete_db_if_empty(&mut tx, &"music/empty/".to_string().try_into()?).await?;

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
        fixtures("delete_folder_with_subfolders")
    )]
    async fn with_subfolders(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        // "music/" フォルダを削除実行（サブフォルダがあるので削除されないはず）
        super::super::delete_db_if_empty(&mut tx, &"music/".to_string().try_into()?).await?;

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
        fixtures("delete_empty_folder_under_root")
    )]
    async fn delete_under_root(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        // "music/" フォルダを削除実行
        super::super::delete_db_if_empty(&mut tx, &"music/".to_string().try_into()?).await?;

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
