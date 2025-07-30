use std::str::FromStr;

use anyhow::Result;
use sqlx::PgPool;

use murack_core_domain::{NonEmptyString, path::LibraryTrackPath};

// track_paths_by_path_str 関数のテスト
mod test_track_paths_by_path_str {

    use super::*;

    /// ディレクトリ指定でのパス取得テスト
    /// 指定されたディレクトリ内の楽曲パス一覧を取得
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/track_paths_by_path_str/directory.sql")
    )]
    async fn ディレクトリ指定(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let result =
            super::super::track_paths_by_path_str(&mut tx, &NonEmptyString::from_str("test/hoge")?)
                .await?;

        // 結果は3つの楽曲パスであるはず
        assert_eq!(result.len(), 3);
        assert!(result.contains(&LibraryTrackPath::from_str("test/hoge/track1.mp3")?));
        assert!(result.contains(&LibraryTrackPath::from_str("test/hoge/track2.flac")?));
        assert!(result.contains(&LibraryTrackPath::from_str("test/hoge/track3.m4a")?));

        Ok(())
    }

    /// 見つからない場合のテスト
    /// 指定されたパスに楽曲が存在しない場合
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/track_paths_by_path_str/not_found.sql")
    )]
    async fn 見つからない場合(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let result =
            super::super::track_paths_by_path_str(&mut tx, &NonEmptyString::from_str("test/hoge")?)
                .await?;

        // 結果は空であるはず
        assert_eq!(result, vec![]);

        Ok(())
    }

    /// 楽曲ファイル指定でのパス取得テスト
    /// 特定の楽曲ファイルを直接指定した場合
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/track_paths_by_path_str/track.sql")
    )]
    async fn 楽曲ファイル指定(pool: PgPool) -> Result<()> {
        let mut tx = pool.begin().await?;

        let result = super::super::track_paths_by_path_str(
            &mut tx,
            &NonEmptyString::from_str("test/hoge.flac")?,
        )
        .await?;

        // 結果は指定した楽曲ファイル1つであるはず
        assert_eq!(result, vec![LibraryTrackPath::from_str("test/hoge.flac")?]);

        // 指定された楽曲がデータベースに存在することを確認
        let exists = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE path = $1"#,
            "test/hoge.flac"
        )
        .fetch_one(&mut *tx)
        .await?;
        assert_eq!(exists, 1);

        Ok(())
    }
}
