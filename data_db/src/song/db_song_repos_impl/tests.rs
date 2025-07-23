use anyhow::Result;
use murack_core_domain::{
    db::DbTransaction,
    path::{LibPathStr, LibSongPath},
    song::DbSongRepository,
};
use sqlx::PgPool;

use crate::song::{DbSongRepositoryImpl, SongDaoImpl};

// get_path_by_path_str 関数のテスト
mod test_get_path_by_path_str {
    use super::*;

    /// ディレクトリ指定でのパス取得テスト
    /// 指定されたディレクトリ内の楽曲パス一覧を取得
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_get_path_by_path_str_directory")
    )]
    async fn ディレクトリ指定(pool: PgPool) -> Result<()> {
        let song_dao = SongDaoImpl {};
        let target = DbSongRepositoryImpl::new(song_dao);

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::from("test/hoge".to_owned()))
            .await?;

        // 結果は3つの楽曲パスであるはず
        assert_eq!(result.len(), 3);
        assert!(result.contains(&LibSongPath::new("test/hoge/song1.mp3")));
        assert!(result.contains(&LibSongPath::new("test/hoge/song2.flac")));
        assert!(result.contains(&LibSongPath::new("test/hoge/song3.m4a")));

        // データベース内の楽曲数を確認
        let track_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE path LIKE 'test/hoge/%'"#
        )
        .fetch_one(&mut **tx.get())
        .await?;
        assert_eq!(track_count, 3);

        Ok(())
    }

    /// 見つからない場合のテスト
    /// 指定されたパスに楽曲が存在しない場合
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_get_path_by_path_str_not_found")
    )]
    async fn 見つからない場合(pool: PgPool) -> Result<()> {
        let song_dao = SongDaoImpl {};
        let target = DbSongRepositoryImpl::new(song_dao);

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::from("test/hoge".to_owned()))
            .await?;

        // 結果は空であるはず
        assert_eq!(result, vec![]);

        // データベース内に楽曲が存在しないことを確認
        let track_count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM tracks"#)
            .fetch_one(&mut **tx.get())
            .await?;
        assert_eq!(track_count, 0);

        Ok(())
    }

    /// 楽曲ファイル指定でのパス取得テスト
    /// 特定の楽曲ファイルを直接指定した場合
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_get_path_by_path_str_song")
    )]
    async fn 楽曲ファイル指定(pool: PgPool) -> Result<()> {
        let song_dao = SongDaoImpl {};
        let target = DbSongRepositoryImpl::new(song_dao);

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::from("test/hoge.flac".to_owned()))
            .await?;

        // 結果は指定した楽曲ファイル1つであるはず
        assert_eq!(result, vec![LibSongPath::new("test/hoge.flac")]);

        // 指定された楽曲がデータベースに存在することを確認
        let exists = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE path = $1"#,
            "test/hoge.flac"
        )
        .fetch_one(&mut **tx.get())
        .await?;
        assert_eq!(exists, 1);

        Ok(())
    }

    /// ルート指定でのパス取得テスト
    /// ライブラリ全体の楽曲を取得する場合
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_get_path_by_path_str_root")
    )]
    async fn ルート指定(pool: PgPool) -> Result<()> {
        let song_dao = SongDaoImpl {};
        let target = DbSongRepositoryImpl::new(song_dao);

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::root())
            .await?;

        // 結果は3つの楽曲パスであるはず
        assert_eq!(result.len(), 3);
        assert!(result.contains(&LibSongPath::new("song1.mp3")));
        assert!(result.contains(&LibSongPath::new("test/hoge/song2.flac")));
        assert!(result.contains(&LibSongPath::new("test/song3.m4a")));

        // データベース内の全楽曲数を確認
        let total_count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM tracks"#)
            .fetch_one(&mut **tx.get())
            .await?;
        assert_eq!(total_count, 3);

        Ok(())
    }
}