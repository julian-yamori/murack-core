use std::str::FromStr;

use anyhow::Result;
use chrono::NaiveDate;
use sqlx::PgPool;

use super::*;

fn target() -> SyncUsecaseImpl {
    SyncUsecaseImpl::new()
}

// register_db 関数のテスト
mod test_register_db {
    use super::*;

    // DB に追加する曲のデータを作成
    fn track_sync() -> TrackSync {
        TrackSync {
            duration: 120000,
            title: "曲名".to_owned(),
            artist: "アーティスト".to_owned(),
            album: "アルバむ".to_owned(),
            genre: "Genre".to_owned(),
            album_artist: "".to_owned(),
            composer: "".to_owned(),
            track_number: Some(1),
            track_max: Some(2),
            disc_number: Some(3),
            disc_max: Some(4),
            release_date: Some(NaiveDate::from_ymd_opt(2013, 7, 14).unwrap()),
            memo: "メモ".to_owned(),
            lyrics: "歌詞".to_owned(),
            // artworks: vec![TrackArtwork {
            //     picture: Arc::new(Picture {
            //         bytes: vec![1, 2, 3, 4],
            //         mime_type: "image/jpeg".to_owned(),
            //     }),
            //     picture_type: 3,
            //     description: "説明".to_owned(),
            // }],

            // アートワークが適当なバイトデータだと mini image の生成に失敗するので、一旦空にしておく
            artworks: vec![],
        }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("register_track_to_root"))]
    async fn to_root_folder(pool: PgPool) -> Result<()> {
        fn track_path() -> LibraryTrackPath {
            LibraryTrackPath::from_str("track.flac").unwrap()
        }

        let target = target();
        let mut tx = pool.begin().await?;

        let s = track_sync();
        target.register_db(&mut tx, &track_path(), &s).await?;

        // 曲がRoot直下（folder_id = NULL）に登録されたことを確認
        let track_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE path = $1 AND folder_id IS NULL"#,
            "track.flac"
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(track_count, 1);

        // 曲の詳細データが正しく登録されたことを確認
        let track = sqlx::query!(
            "SELECT title, artist, album, duration FROM tracks WHERE path = $1",
            "track.flac"
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(track.title, "曲名");
        assert_eq!(track.artist, "アーティスト");
        assert_eq!(track.album, "アルバむ");
        assert_eq!(track.duration, 120000);

        // フォルダは作成されていないことを確認
        let folder_count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM folder_paths"#)
            .fetch_one(&mut *tx)
            .await?;
        assert_eq!(folder_count, 0);

        // プレイリストのlistuped_flagがリセットされたことを確認
        let listuped_true_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM playlists WHERE listuped_flag = true"#
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(listuped_true_count, 0);

        // // アートワークが登録されたことを確認
        // let artwork_count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM artworks"#)
        //     .fetch_one(&mut *tx)
        //     .await?;
        // assert_eq!(artwork_count, 1);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("register_track_to_folder"))]
    async fn in_folder(pool: PgPool) -> Result<()> {
        fn track_path() -> LibraryTrackPath {
            LibraryTrackPath::from_str("test/hoge/fuga.mp3").unwrap()
        }

        let target = target();
        let mut tx = pool.begin().await?;

        let s = track_sync();
        target.register_db(&mut tx, &track_path(), &s).await?;

        // フォルダが作成されたことを確認
        let test_folder_id =
            sqlx::query_scalar!("SELECT id FROM folder_paths WHERE path = $1", "test/")
                .fetch_optional(&mut *tx)
                .await?;
        assert!(test_folder_id.is_some());

        let hoge_folder_id =
            sqlx::query_scalar!("SELECT id FROM folder_paths WHERE path = $1", "test/hoge/")
                .fetch_optional(&mut *tx)
                .await?;
        assert!(hoge_folder_id.is_some());

        // 曲が正しいフォルダIDで登録されたことを確認
        let track_folder_id = sqlx::query_scalar!(
            "SELECT folder_id FROM tracks WHERE path = $1",
            "test/hoge/fuga.mp3"
        )
        .fetch_optional(&mut *tx)
        .await?;
        assert_eq!(track_folder_id, Some(hoge_folder_id));

        // 曲の詳細データが正しく登録されたことを確認
        let track = sqlx::query!(
            "SELECT title, artist, album, duration FROM tracks WHERE path = $1",
            "test/hoge/fuga.mp3"
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(track.title, "曲名");
        assert_eq!(track.artist, "アーティスト");
        assert_eq!(track.album, "アルバむ");
        assert_eq!(track.duration, 120000);

        // プレイリストのlistuped_flagがリセットされたことを確認
        let listuped_true_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM playlists WHERE listuped_flag = true"#
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(listuped_true_count, 0);

        // // アートワークが登録されたことを確認
        // let artwork_count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM artworks"#)
        //     .fetch_one(&mut *tx)
        //     .await?;
        // assert_eq!(artwork_count, 1);

        Ok(())
    }
}
