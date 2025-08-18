use anyhow::Result;
use sqlx::PgPool;
use std::str::FromStr;

use crate::{
    path::LibraryTrackPath,
    track_query::{SelectColumn, playlist_query::PlaylistQueryBuilder},
};

/// PlaylistQuery::fetch() のテスト
mod test_fetch {
    use super::*;

    /// ID カラムのみ取得テスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_basic"))]
    async fn test_fetch_id_only(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::Id)
            .build();

        let rows = query.fetch(&mut tx).await?;

        assert_eq!(rows.len(), 3);

        let ids: Vec<i32> = rows
            .iter()
            .map(|row| SelectColumn::row_id(row).unwrap())
            .collect();

        // プレイリストの並び順で取得される
        assert_eq!(ids, vec![1, 2, 3]);

        Ok(())
    }

    /// Path カラム取得テスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_basic"))]
    async fn test_fetch_path(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        // Playlist 2 の方を取得してみる
        let query = PlaylistQueryBuilder::new(2)
            .column(SelectColumn::Path)
            .build();

        let rows = query.fetch(&mut tx).await?;

        assert_eq!(rows.len(), 2);

        let paths: Vec<LibraryTrackPath> = rows
            .iter()
            .map(|row| SelectColumn::row_path(row).unwrap())
            .collect();

        assert_eq!(
            paths,
            vec![
                LibraryTrackPath::from_str("/music/track2.mp3").unwrap(),
                LibraryTrackPath::from_str("/music/track4.mp3").unwrap(),
            ]
        );

        Ok(())
    }

    /// Title カラム取得テスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_basic"))]
    async fn test_fetch_title(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::Title)
            .build();

        let rows = query.fetch(&mut tx).await?;

        assert_eq!(rows.len(), 3);

        let titles: Vec<String> = rows
            .iter()
            .map(|row| SelectColumn::row_title(row).unwrap())
            .collect();

        assert_eq!(titles, vec!["Track A", "Track B", "Track C"]);

        Ok(())
    }

    /// ArtworkId カラム取得テスト（特別な JOIN が発生）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_artwork"))]
    async fn test_fetch_artwork_id(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::ArtworkId)
            .build();

        let rows = query.fetch(&mut tx).await?;

        assert_eq!(rows.len(), 3);

        let artwork_ids: Vec<Option<i32>> = rows
            .iter()
            .map(|row| SelectColumn::row_artwork_id(row).unwrap())
            .collect();

        // track1: artwork_id = 1, track2: None, track3: artwork_id = 2
        assert_eq!(artwork_ids, vec![Some(1), None, Some(2)]);

        Ok(())
    }

    /// 全カラムの組み合わせ取得テスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_artwork"))]
    async fn test_fetch_all_columns(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::Id)
            .column(SelectColumn::Path)
            .column(SelectColumn::Title)
            .column(SelectColumn::ArtworkId)
            .build();

        let rows = query.fetch(&mut tx).await?;

        assert_eq!(rows.len(), 3);

        // 最初のレコードのデータを確認
        let first_row = &rows[0];
        assert_eq!(SelectColumn::row_id(first_row)?, 1);
        assert_eq!(
            SelectColumn::row_path(first_row)?,
            LibraryTrackPath::from_str("/music/track1.mp3").unwrap()
        );
        assert_eq!(SelectColumn::row_title(first_row)?, "Track A");
        assert_eq!(SelectColumn::row_artwork_id(first_row)?, Some(1));

        Ok(())
    }

    /// 複数カラム（Id + Title）の組み合わせテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_basic"))]
    async fn test_fetch_id_and_title(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::Id)
            .column(SelectColumn::Title)
            .build();

        let rows = query.fetch(&mut tx).await?;

        assert_eq!(rows.len(), 3);

        for (i, row) in rows.iter().enumerate() {
            let id = SelectColumn::row_id(row)?;
            let title = SelectColumn::row_title(row)?;

            match i {
                0 => {
                    assert_eq!(id, 1);
                    assert_eq!(title, "Track A");
                }
                1 => {
                    assert_eq!(id, 2);
                    assert_eq!(title, "Track B");
                }
                2 => {
                    assert_eq!(id, 3);
                    assert_eq!(title, "Track C");
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }
}

/// プレイリストタイプ別のテスト
mod test_playlist_types {
    use super::*;

    /// Normal プレイリストのテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_basic"))]
    async fn test_normal_playlist(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1) // Normal プレイリスト
            .column(SelectColumn::Id)
            .build();

        let rows = query.fetch(&mut tx).await?;
        let ids: Vec<i32> = rows
            .iter()
            .map(|row| SelectColumn::row_id(row).unwrap())
            .collect();

        // playlist_tracks の order_index 順
        assert_eq!(ids, vec![1, 2, 3]);

        Ok(())
    }

    /// Filter プレイリストのテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_filter"))]
    async fn test_filter_playlist(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(2) // Filter プレイリスト
            .column(SelectColumn::Id)
            .build();

        let rows = query.fetch(&mut tx).await?;
        let ids: Vec<i32> = rows
            .iter()
            .map(|row| SelectColumn::row_id(row).unwrap())
            .collect();

        // フィルタ条件にマッチする曲（rating >= 4）
        assert_eq!(ids, vec![1, 3]);

        Ok(())
    }

    /// Folder プレイリストのテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_folder"))]
    async fn test_folder_playlist(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(3) // Folder プレイリスト
            .column(SelectColumn::Id)
            .build();

        let rows = query.fetch(&mut tx).await?;
        let ids: Vec<i32> = rows
            .iter()
            .map(|row| SelectColumn::row_id(row).unwrap())
            .collect();

        // 子プレイリスト（プレイリスト4, 5）の曲を集約
        // 重複は排除される
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
        assert!(ids.contains(&4));

        Ok(())
    }
}

/// ソート機能のテスト
mod test_sorting {
    use super::*;

    /// プレイリスト順ソート（昇順）のテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_sort"))]
    async fn test_playlist_sort_asc(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::Id)
            .build();

        let rows = query.fetch(&mut tx).await?;
        let ids: Vec<i32> = rows
            .iter()
            .map(|row| SelectColumn::row_id(row).unwrap())
            .collect();

        // プレイリストの order_index 順
        assert_eq!(ids, vec![3, 1, 2]);

        Ok(())
    }

    /// プレイリスト順ソート（降順）のテスト
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_playlist_query_sort_desc")
    )]
    async fn test_playlist_sort_desc(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::Id)
            .build();

        let rows = query.fetch(&mut tx).await?;
        let ids: Vec<i32> = rows
            .iter()
            .map(|row| SelectColumn::row_id(row).unwrap())
            .collect();

        // プレイリストの order_index 降順
        assert_eq!(ids, vec![2, 1, 3]);

        Ok(())
    }

    /// Artist ソートのテスト
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_playlist_query_artist_sort")
    )]
    async fn test_artist_sort(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::Id)
            .build();

        let rows = query.fetch(&mut tx).await?;
        let ids: Vec<i32> = rows
            .iter()
            .map(|row| SelectColumn::row_id(row).unwrap())
            .collect();

        // artist_order ASC でソート
        assert_eq!(ids, vec![1, 3, 2]);

        Ok(())
    }
}

/// ページネーション機能のテスト
mod test_pagination {
    use super::*;

    /// LIMIT のみのテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_basic"))]
    async fn test_limit_only(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::Id)
            .limit(2)
            .build();

        let rows = query.fetch(&mut tx).await?;

        assert_eq!(rows.len(), 2);
        let ids: Vec<i32> = rows
            .iter()
            .map(|row| SelectColumn::row_id(row).unwrap())
            .collect();
        assert_eq!(ids, vec![1, 2]);

        Ok(())
    }

    /// OFFSET のみのテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_basic"))]
    async fn test_offset_only(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::Id)
            .offset(1)
            .build();

        let rows = query.fetch(&mut tx).await?;

        assert_eq!(rows.len(), 2);
        let ids: Vec<i32> = rows
            .iter()
            .map(|row| SelectColumn::row_id(row).unwrap())
            .collect();
        assert_eq!(ids, vec![2, 3]);

        Ok(())
    }

    /// LIMIT + OFFSET のテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_basic"))]
    async fn test_limit_and_offset(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1)
            .column(SelectColumn::Id)
            .limit(1)
            .offset(1)
            .build();

        let rows = query.fetch(&mut tx).await?;

        assert_eq!(rows.len(), 1);
        let ids: Vec<i32> = rows
            .iter()
            .map(|row| SelectColumn::row_id(row).unwrap())
            .collect();
        assert_eq!(ids, vec![2]);

        Ok(())
    }
}

/// エラーハンドリングのテスト
mod test_error_handling {
    use super::*;

    /// 存在しないプレイリスト ID のテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_basic"))]
    async fn test_nonexistent_playlist_id(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(999) // 存在しない ID
            .column(SelectColumn::Id)
            .build();

        let result = query.fetch(&mut tx).await;

        assert!(result.is_err());

        Ok(())
    }

    /// Filter プレイリストでフィルタが null のテスト
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_playlist_query_filter_null")
    )]
    async fn test_filter_playlist_null_filter(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(2) // filter_json が null の Filter プレイリスト
            .column(SelectColumn::Id)
            .build();

        let result = query.fetch(&mut tx).await;

        assert!(result.is_err());

        Ok(())
    }

    /// 空のプレイリストのテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_playlist_query_empty"))]
    async fn test_empty_playlist(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let query = PlaylistQueryBuilder::new(1) // 曲が含まれていないプレイリスト
            .column(SelectColumn::Id)
            .build();

        let rows = query.fetch(&mut tx).await?;

        assert_eq!(rows.len(), 0);

        Ok(())
    }
}
