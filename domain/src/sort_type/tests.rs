use anyhow::Result;
use sqlx::PgPool;

use crate::{SortType, SortTypeWithPlaylist};

/// SortType::order_query() のテスト
mod test_order_query {
    use super::*;

    /// TrackNameソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_track_name(pool: PgPool) -> Result<()> {
        let order_query = SortType::TrackName.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // title_order ASC, tracks.id ASC でソートされる
        // A Song(2), B Song(3), C Song(1), D Song(4), E Song(5)
        assert_eq!(actual, vec![2, 3, 1, 4, 5]);

        Ok(())
    }

    /// TrackNameソート（降順）のテスト
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_track_name_desc(pool: PgPool) -> Result<()> {
        let order_query = SortType::TrackName.order_query(true);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // title_order DESC, tracks.id DESC でソート
        // E Song(5), D Song(4), C Song(1), B Song(3), A Song(2)
        assert_eq!(actual, vec![5, 4, 1, 3, 2]);

        Ok(())
    }

    /// Artistソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_artist(pool: PgPool) -> Result<()> {
        let order_query = SortType::Artist.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // artist_order ASC, album_order ASC, disc_number ASC, track_number ASC でソート
        // Artist A: Album A(2, 4), Artist B: Album B(5), Album Z(1), Artist C: Album B(3)
        assert_eq!(actual, vec![2, 4, 5, 1, 3]);

        Ok(())
    }

    /// Albumソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_album(pool: PgPool) -> Result<()> {
        let order_query = SortType::Album.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // album_order ASC, artist_order ASC, disc_number ASC, track_number ASC でソート
        // Album A: Artist A(2, 4), Album B: Artist B(5), Artist C(3), Album Z: Artist B(1)
        assert_eq!(actual, vec![2, 4, 5, 3, 1]);

        Ok(())
    }

    /// Genreソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_genre(pool: PgPool) -> Result<()> {
        let order_query = SortType::Genre.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        assert_eq!(actual, vec![2, 5, 3, 4, 1]);

        Ok(())
    }

    /// Composerソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_composer(pool: PgPool) -> Result<()> {
        let order_query = SortType::Composer.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // composer_order ASC, artist_order ASC, album_order ASC, disc_number ASC, track_number ASC でソート
        // Composer A: Artist A Album A(1, 4), Artist B Album Z(1), Composer B: Artist A Album A(2), Artist B Album B(5), Composer C: Artist C Album B(3)
        assert_eq!(actual, vec![4, 1, 2, 5, 3]);

        Ok(())
    }

    /// Durationソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_duration(pool: PgPool) -> Result<()> {
        let order_query = SortType::Duration.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // duration ASC, title_order ASC, tracks.id ASC でソート
        // 160(4), 180(1), 200(3), 220(5), 240(2)
        assert_eq!(actual, vec![4, 1, 3, 5, 2]);

        Ok(())
    }

    /// TrackIndexソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_track_index(pool: PgPool) -> Result<()> {
        let order_query = SortType::TrackIndex.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // track_number ASC, artist_order ASC, album_order ASC でソート
        // track 1: (1, 3), track 2: (2, 5), track 3: (4)
        assert_eq!(actual, vec![1, 3, 2, 5, 4]);

        Ok(())
    }

    /// DiscIndexソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_disc_index(pool: PgPool) -> Result<()> {
        let order_query = SortType::DiscIndex.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // disc_number ASC, artist_order ASC, album_order ASC, track_number ASC でソート
        // disc 1: Artist A Album A (1, 4), Artist B Album Z (1), disc 2: Artist B Album B (5), Artist C Album B (3)
        assert_eq!(actual, vec![2, 4, 1, 5, 3]);

        Ok(())
    }

    /// ReleaseDateソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_release_date(pool: PgPool) -> Result<()> {
        let order_query = SortType::ReleaseDate.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // release_date ASC, artist_order ASC, album_order ASC でソート
        // 2023-01-01(1), 2023-01-15(4), 2023-02-01(2), 2023-02-15(5), 2023-03-01(3)
        assert_eq!(actual, vec![1, 4, 2, 5, 3]);

        Ok(())
    }

    /// Ratingソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_rating(pool: PgPool) -> Result<()> {
        let order_query = SortType::Rating.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // rating ASC, artist_order ASC, album_order ASC でソート
        // rating 2(4), rating 3(2), rating 4(3), rating 5(1, 5)
        assert_eq!(actual, vec![4, 2, 3, 5, 1]);

        Ok(())
    }

    /// EntryDateソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_entry_date(pool: PgPool) -> Result<()> {
        let order_query = SortType::EntryDate.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // created_at ASC, path ASC でソート
        // 2023-06-01(1), 2023-06-02(2), 2023-06-03(3), 2023-06-04(4), 2023-06-05(5)
        assert_eq!(actual, vec![1, 2, 3, 4, 5]);

        Ok(())
    }

    /// Pathソートのテスト（昇順）
    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_sort_order"))]
    async fn test_path(pool: PgPool) -> Result<()> {
        let order_query = SortType::Path.order_query(false);
        let sql = format!("SELECT id FROM tracks ORDER BY {order_query}");

        let actual: Vec<i32> = sqlx::query_scalar(&sql).fetch_all(&pool).await?;

        // path ASC でソート
        // /music/album1/01.mp3(1), /music/album1/04.mp3(4), /music/album2/02.mp3(2), /music/album2/05.mp3(5), /music/album3/03.mp3(3)
        assert_eq!(actual, vec![1, 4, 2, 5, 3]);

        Ok(())
    }
}

/// SortTypeWithPlaylist::order_query() のテスト
mod test_order_query_with_playlist {
    use sqlx::{Row, postgres::PgRow};

    use super::*;

    const PLAYLIST_ORDER_COLUMN: &str = "pt.order_index";

    /// プレイリストの曲を指定されたカラムでソートし、id を取得
    async fn select_track_ids(pool: &PgPool, order_query: &str) -> anyhow::Result<Vec<i32>> {
        let sql = format!(
            "
            SELECT tracks.id, {PLAYLIST_ORDER_COLUMN}
            FROM playlist_tracks AS pt
            LEFT JOIN tracks ON pt.track_id = tracks.id
            WHERE pt.playlist_id = 1
            ORDER BY {order_query}
            "
        );

        let ids = sqlx::query(&sql)
            .map(|row: PgRow| row.get::<i32, _>("id"))
            .fetch_all(pool)
            .await?;

        Ok(ids)
    }

    /// Artistソートのテスト（昇順）
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_sort_order_with_playlist")
    )]
    async fn test_artist(pool: PgPool) -> Result<()> {
        let sort_type = SortTypeWithPlaylist::General(SortType::Artist);
        let order_query = sort_type.order_query(false, PLAYLIST_ORDER_COLUMN);

        let actual = select_track_ids(&pool, &order_query).await?;

        // プレイリストに含まれる曲を artist_order ASC, album_order ASC, disc_number ASC, track_number ASC でソート
        // Artist A: Album A(2, 4), Artist B: Album B(5), Album Z(1)
        assert_eq!(actual, vec![2, 4, 5, 1]);

        Ok(())
    }

    /// Artistソートのテスト（降順）
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_sort_order_with_playlist")
    )]
    async fn test_artist_desc(pool: PgPool) -> Result<()> {
        let sort_type = SortTypeWithPlaylist::General(SortType::Artist);
        let order_query = sort_type.order_query(true, PLAYLIST_ORDER_COLUMN);

        let actual = select_track_ids(&pool, &order_query).await?;

        // プレイリストに含まれる曲を artist_order DESC, album_order DESC, disc_number DESC, track_number DESC でソート
        assert_eq!(actual, vec![1, 5, 4, 2]);

        Ok(())
    }

    /// Playlistソートのテスト（昇順）
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_sort_order_with_playlist")
    )]
    async fn test_playlist_asc(pool: PgPool) -> Result<()> {
        let sort_type = SortTypeWithPlaylist::Playlist;
        let order_query = sort_type.order_query(false, PLAYLIST_ORDER_COLUMN);

        let actual = select_track_ids(&pool, &order_query).await?;

        // プレイリストに含まれる曲を、プレイリストの並び順でソート
        assert_eq!(actual, vec![1, 5, 2, 4]);

        Ok(())
    }

    /// Playlistソートのテスト（降順）
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_sort_order_with_playlist")
    )]
    async fn test_playlist_desc(pool: PgPool) -> Result<()> {
        let sort_type = SortTypeWithPlaylist::Playlist;
        let order_query = sort_type.order_query(true, PLAYLIST_ORDER_COLUMN);

        let actual = select_track_ids(&pool, &order_query).await?;

        // プレイリストに含まれる曲を、プレイリストの並び順でソート
        assert_eq!(actual, vec![4, 2, 5, 1]);

        Ok(())
    }
}
