use anyhow::Result;
use sqlx::PgPool;

use super::*;
use crate::{folder::DbFolderRepositoryImpl, track::DbTrackRepositoryImpl};

fn target() -> FolderUsecaseImpl<DbFolderRepositoryImpl, DbTrackRepositoryImpl> {
    FolderUsecaseImpl::new(DbFolderRepositoryImpl::new(), DbTrackRepositoryImpl::new())
}

// delete_db_if_empty_by_id 関数のテスト
mod test_delete_db_if_empty_by_id {
    use super::*;

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("delete_empty_with_parent_has_tracks")
    )]
    async fn parent_has_tracks(pool: PgPool) -> Result<()> {
        let target = target();
        let mut tx = pool.begin().await?;

        // フォルダ15を削除実行
        target.delete_db_if_empty_by_id(&mut tx, 15).await?;

        // フォルダ15が削除されたことを確認
        let folder_15_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE id = $1"#,
            15
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(folder_15_count, 0);

        // フォルダ4は削除されていないことを確認（曲があるため）
        let folder_4_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE id = $1"#,
            4
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(folder_4_count, 1);

        // 曲は残っていることを確認
        let track_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE folder_id = $1"#,
            4
        )
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
        let target = target();
        let mut tx = pool.begin().await?;

        // フォルダ15を削除実行（サブフォルダがあるので削除されないはず）
        target.delete_db_if_empty_by_id(&mut tx, 15).await?;

        // フォルダ15が削除されていないことを確認
        let folder_15_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE id = $1"#,
            15
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(folder_15_count, 1);

        // サブフォルダも残っていることを確認
        let subfolder_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE id = $1"#,
            20
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(subfolder_count, 1);

        Ok(())
    }

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("delete_empty_folder_under_root")
    )]
    async fn delete_under_root(pool: PgPool) -> Result<()> {
        let target = target();
        let mut tx = pool.begin().await?;

        // フォルダ15を削除実行
        target.delete_db_if_empty_by_id(&mut tx, 15).await?;

        // フォルダ15が削除されたことを確認
        let folder_15_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE id = $1"#,
            15
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(folder_15_count, 0);

        // 無関係なフォルダは残っていることを確認
        let remain_id =
            sqlx::query_scalar!("SELECT id FROM folder_paths WHERE path = 'otherfolder/'")
                .fetch_optional(&mut *tx)
                .await?;
        assert!(remain_id.is_some());

        Ok(())
    }
}
