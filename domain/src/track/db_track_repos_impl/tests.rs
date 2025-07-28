use std::str::FromStr;

use anyhow::Result;
use sqlx::PgPool;

use crate::{
    NonEmptyString,
    path::{LibraryDirectoryPath, LibraryTrackPath},
    track::{DbTrackRepository, DbTrackRepositoryImpl},
};

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
        let target = DbTrackRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        let result = target
            .get_path_by_path_str(&mut tx, &NonEmptyString::from_str("test/hoge")?)
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
        fixtures("test_get_path_by_path_str_not_found")
    )]
    async fn 見つからない場合(pool: PgPool) -> Result<()> {
        let target = DbTrackRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        let result = target
            .get_path_by_path_str(&mut tx, &NonEmptyString::from_str("test/hoge")?)
            .await?;

        // 結果は空であるはず
        assert_eq!(result, vec![]);

        Ok(())
    }

    /// 楽曲ファイル指定でのパス取得テスト
    /// 特定の楽曲ファイルを直接指定した場合
    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("test_get_path_by_path_str_track")
    )]
    async fn 楽曲ファイル指定(pool: PgPool) -> Result<()> {
        let target = DbTrackRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        let result = target
            .get_path_by_path_str(&mut tx, &NonEmptyString::from_str("test/hoge.flac")?)
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

// get_path_by_directory 関数のテスト
mod test_get_path_by_directory {
    use super::*;

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/test_get_path_by_directory/normal_chars.sql")
    )]
    fn 浅いディレクトリを指定(pool: PgPool) -> anyhow::Result<()> {
        let target = DbTrackRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        assert_eq!(
            target
                .get_path_by_directory(&mut tx, &LibraryDirectoryPath::from_str("test")?)
                .await?,
            vec![
                LibraryTrackPath::from_str("test/hoge.flac")?,
                LibraryTrackPath::from_str("test/hoge2.flac")?,
                LibraryTrackPath::from_str("test/dir/hoge3.flac")?,
            ]
        );

        Ok(())
    }

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/test_get_path_by_directory/normal_chars.sql")
    )]
    fn 少し深いディレクトリを指定(pool: PgPool) -> anyhow::Result<()> {
        let target = DbTrackRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        assert_eq!(
            target
                .get_path_by_directory(&mut tx, &LibraryDirectoryPath::from_str("test/dir")?)
                .await?,
            vec![LibraryTrackPath::from_str("test/dir/hoge3.flac")?]
        );

        Ok(())
    }

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/test_get_path_by_directory/special_chars.sql")
    )]
    fn 特殊文字を挟むパスでの検索(pool: PgPool) -> anyhow::Result<()> {
        let target = DbTrackRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        assert_eq!(
            target
                .get_path_by_directory(&mut tx, &LibraryDirectoryPath::from_str("test/d%i_r$")?)
                .await?,
            vec![LibraryTrackPath::from_str("test/d%i_r$/hoge.flac")?]
        );

        Ok(())
    }
}

// is_exist_in_folder 関数のテスト
mod test_is_exist_in_folder {
    use super::*;

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_is_exist_in_folder"))]
    async fn フォルダに2曲存在する場合(pool: PgPool) -> anyhow::Result<()> {
        let target = DbTrackRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        // フォルダID 11 には2曲存在するため true
        assert!(target.is_exist_in_folder(&mut tx, 11).await?);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_is_exist_in_folder"))]
    async fn フォルダに1曲だけ存在する場合(pool: PgPool) -> anyhow::Result<()> {
        let target = DbTrackRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        // フォルダID 22 には1曲存在するため true
        assert!(target.is_exist_in_folder(&mut tx, 22).await?);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_is_exist_in_folder"))]
    async fn フォルダに曲が存在しない場合(pool: PgPool) -> anyhow::Result<()> {
        let target = DbTrackRepositoryImpl::new();

        let mut tx = pool.begin().await?;

        // フォルダID 99 には曲が存在しないため false
        assert!(!target.is_exist_in_folder(&mut tx, 99).await?);

        Ok(())
    }
}
