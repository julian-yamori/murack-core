// PlaylistTree::from_all_playlists 関数のテスト
mod test_from_all_playlists {
    use sqlx::{PgPool, PgTransaction};

    use crate::{
        NonEmptyString,
        playlist::{PlaylistTree, PlaylistTreeValue},
    };

    /// テストで使用する PlaylistTreeValue 実装
    #[derive(Debug, PartialEq, Eq)]
    struct Playlist {
        /// プレイリストID
        pub id: i32,

        /// プレイリスト名
        pub name: NonEmptyString,

        /// 親プレイリストID
        pub parent_id: Option<i32>,

        /// 親プレイリスト内でのインデックス
        pub in_folder_order: i32,
    }

    impl Playlist {
        async fn get_all<'c>(tx: &mut PgTransaction<'c>) -> sqlx::Result<Vec<Self>> {
            sqlx::query_as!(Self, r#"SELECT id, name AS "name: NonEmptyString", parent_id, in_folder_order FROM playlists"#)
                .fetch_all(&mut **tx)
                .await
        }
    }

    impl PlaylistTreeValue for Playlist {
        fn id(&self) -> i32 {
            self.id
        }

        fn parent_id(&self) -> Option<i32> {
            self.parent_id
        }

        fn in_folder_order(&self) -> i32 {
            self.in_folder_order
        }
    }

    /// 空のプレイリストツリーを取得するテスト
    /// データベースにプレイリストが存在しない場合
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_get_whole_tree_empty"))]
    async fn 空の場合(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;
        let all_playlist = Playlist::get_all(&mut tx).await?;

        let result = PlaylistTree::from_all_playlists(all_playlist)?;

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
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_get_whole_tree_flat"))]
    async fn フラット構造(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;
        let all_playlist = Playlist::get_all(&mut tx).await?;

        let result = PlaylistTree::from_all_playlists(all_playlist)?;

        // 結果は3つのルートプレイリストであるはず
        assert_eq!(result.len(), 3);

        // 各プレイリストに子がないことを確認
        for tree in &result {
            assert_eq!(tree.children.len(), 0);
            assert_eq!(tree.value.parent_id, None);
        }

        // プレイリスト名を確認
        let names: Vec<&str> = result.iter().map(|t| t.value.name.as_ref()).collect();
        assert_eq!(names, vec!["one", "two", "three"]);

        // プレイリストIDを確認
        let ids: Vec<i32> = result.iter().map(|t| t.value.id).collect();
        assert_eq!(ids, vec![3, 5, 2]);

        Ok(())
    }

    /// 階層構造のあるプレイリストツリーのテスト
    /// 複雑な親子関係を持つプレイリストが存在する場合
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_get_whole_tree_treed"))]
    async fn 階層構造(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;
        let all_playlist = Playlist::get_all(&mut tx).await?;

        let result = PlaylistTree::from_all_playlists(all_playlist)?;

        // 結果は2つのルートプレイリスト（root1, root2）であるはず
        assert_eq!(result.len(), 2);

        // root1 (id=5) の確認
        let root1 = &result[0];
        assert_eq!(root1.value.id, 5);
        assert_eq!(root1.value.name, "root1".to_string().try_into()?);
        assert_eq!(root1.value.parent_id, None);
        assert_eq!(root1.children.len(), 3); // 1-1, 1-2, 1-3

        // root1 の子プレイリスト確認
        let child_1_1 = &root1.children[0];
        assert_eq!(child_1_1.value.id, 3);
        assert_eq!(child_1_1.value.name, "1-1".to_string().try_into()?);
        assert_eq!(child_1_1.value.parent_id, Some(5));
        assert_eq!(child_1_1.children.len(), 0);

        let child_1_2 = &root1.children[1];
        assert_eq!(child_1_2.value.id, 2);
        assert_eq!(child_1_2.value.name, "1-2".to_string().try_into()?);
        assert_eq!(child_1_2.value.parent_id, Some(5));
        assert_eq!(child_1_2.children.len(), 2); // 1-2-1, 1-2-2

        // 1-2 の子プレイリスト確認
        let child_1_2_1 = &child_1_2.children[0];
        assert_eq!(child_1_2_1.value.id, 9);
        assert_eq!(child_1_2_1.value.name, "1-2-1".to_string().try_into()?);
        assert_eq!(child_1_2_1.value.parent_id, Some(2));

        let child_1_2_2 = &child_1_2.children[1];
        assert_eq!(child_1_2_2.value.id, 98);
        assert_eq!(child_1_2_2.value.name, "1-2-2".to_string().try_into()?);
        assert_eq!(child_1_2_2.value.parent_id, Some(2));

        // root2 (id=35) の確認
        let root2 = &result[1];
        assert_eq!(root2.value.id, 35);
        assert_eq!(root2.value.name, "root2".to_string().try_into()?);
        assert_eq!(root2.value.parent_id, None);
        assert_eq!(root2.children.len(), 1); // 2-1

        let child_2_1 = &root2.children[0];
        assert_eq!(child_2_1.value.id, 75);
        assert_eq!(child_2_1.value.name, "2-1".to_string().try_into()?);
        assert_eq!(child_2_1.value.parent_id, Some(35));
        assert_eq!(child_2_1.children.len(), 0);

        Ok(())
    }
}
