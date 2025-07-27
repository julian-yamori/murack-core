use anyhow::Result;
use murack_core_domain::playlist::DbPlaylistRepository;
use sqlx::PgPool;

use crate::playlist::DbPlaylistRepositoryImpl;

// get_playlist_tree 関数のテスト
mod test_get_playlist_tree {
    use super::*;

    /// 空のプレイリストツリーを取得するテスト
    /// データベースにプレイリストが存在しない場合
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_get_playlist_tree_empty"))]
    async fn 空の場合(pool: PgPool) -> Result<()> {
        let target = DbPlaylistRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        let result = target.get_playlist_tree(&mut tx).await?;

        // 結果は空のベクタであるはず
        assert_eq!(result, vec![]);

        // データベースにプレイリストが存在しないことを確認
        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM playlists"#)
            .fetch_one(&mut *tx)
            .await?;
        assert_eq!(count, 0);

        Ok(())
    }

    /// フラットなプレイリスト構造のテスト
    /// 親子関係のない3つのプレイリストが存在する場合
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_get_playlist_tree_flat"))]
    async fn フラット構造(pool: PgPool) -> Result<()> {
        let target = DbPlaylistRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        let result = target.get_playlist_tree(&mut tx).await?;

        // 結果は3つのルートプレイリストであるはず
        assert_eq!(result.len(), 3);

        // 各プレイリストに子がないことを確認
        for playlist in &result {
            assert_eq!(playlist.children.len(), 0);
            assert_eq!(playlist.parent_id, None);
            assert_eq!(playlist.parent_names.len(), 0);
        }

        // プレイリスト名を確認
        let names: Vec<&str> = result.iter().map(|p| p.name.as_ref()).collect();
        assert_eq!(names, vec!["one", "two", "three"]);

        // プレイリストIDを確認
        let ids: Vec<i32> = result.iter().map(|p| p.rowid).collect();
        assert_eq!(ids, vec![3, 5, 2]);

        Ok(())
    }

    /// 階層構造のあるプレイリストツリーのテスト
    /// 複雑な親子関係を持つプレイリストが存在する場合
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_get_playlist_tree_treed"))]
    async fn 階層構造(pool: PgPool) -> Result<()> {
        let target = DbPlaylistRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        let result = target.get_playlist_tree(&mut tx).await?;

        // 結果は2つのルートプレイリスト（root1, root2）であるはず
        assert_eq!(result.len(), 2);

        // root1 (id=5) の確認
        let root1 = &result[0];
        assert_eq!(root1.rowid, 5);
        assert_eq!(root1.name, "root1".to_string().try_into()?);
        assert_eq!(root1.parent_id, None);
        assert_eq!(root1.parent_names.len(), 0);
        assert_eq!(root1.children.len(), 3); // 1-1, 1-2, 1-3

        // root1 の子プレイリスト確認
        let child_1_1 = &root1.children[0];
        assert_eq!(child_1_1.rowid, 3);
        assert_eq!(child_1_1.name, "1-1".to_string().try_into()?);
        assert_eq!(child_1_1.parent_id, Some(5));
        assert_eq!(
            child_1_1.parent_names,
            vec!["root1".to_string().try_into()?]
        );
        assert_eq!(child_1_1.children.len(), 0);

        let child_1_2 = &root1.children[1];
        assert_eq!(child_1_2.rowid, 2);
        assert_eq!(child_1_2.name, "1-2".to_string().try_into()?);
        assert_eq!(child_1_2.parent_id, Some(5));
        assert_eq!(
            child_1_2.parent_names,
            vec!["root1".to_string().try_into()?]
        );
        assert_eq!(child_1_2.children.len(), 2); // 1-2-1, 1-2-2

        // 1-2 の子プレイリスト確認
        let child_1_2_1 = &child_1_2.children[0];
        assert_eq!(child_1_2_1.rowid, 9);
        assert_eq!(child_1_2_1.name, "1-2-1".to_string().try_into()?);
        assert_eq!(child_1_2_1.parent_id, Some(2));
        assert_eq!(
            child_1_2_1.parent_names,
            vec![
                "root1".to_string().try_into()?,
                "1-2".to_string().try_into()?
            ]
        );

        let child_1_2_2 = &child_1_2.children[1];
        assert_eq!(child_1_2_2.rowid, 98);
        assert_eq!(child_1_2_2.name, "1-2-2".to_string().try_into()?);
        assert_eq!(child_1_2_2.parent_id, Some(2));
        assert_eq!(
            child_1_2_2.parent_names,
            vec![
                "root1".to_string().try_into()?,
                "1-2".to_string().try_into()?
            ]
        );

        // root2 (id=35) の確認
        let root2 = &result[1];
        assert_eq!(root2.rowid, 35);
        assert_eq!(root2.name, "root2".to_string().try_into()?);
        assert_eq!(root2.parent_id, None);
        assert_eq!(root2.parent_names.len(), 0);
        assert_eq!(root2.children.len(), 1); // 2-1

        let child_2_1 = &root2.children[0];
        assert_eq!(child_2_1.rowid, 75);
        assert_eq!(child_2_1.name, "2-1".to_string().try_into()?);
        assert_eq!(child_2_1.parent_id, Some(35));
        assert_eq!(
            child_2_1.parent_names,
            vec!["root2".to_string().try_into()?]
        );
        assert_eq!(child_2_1.children.len(), 0);

        Ok(())
    }
}
