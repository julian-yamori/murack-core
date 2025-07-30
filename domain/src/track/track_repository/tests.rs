use std::str::FromStr;

use sqlx::PgPool;

use crate::path::{LibraryDirectoryPath, LibraryTrackPath};

// get_path_by_directory 関数のテスト
mod test_get_path_by_directory {
    use super::*;

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/test_get_path_by_directory/normal_chars.sql")
    )]
    fn 浅いディレクトリを指定(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        assert_eq!(
            super::super::get_path_by_directory(&mut tx, &LibraryDirectoryPath::from_str("test")?)
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
        let mut tx = pool.begin().await?;

        assert_eq!(
            super::super::get_path_by_directory(
                &mut tx,
                &LibraryDirectoryPath::from_str("test/dir")?
            )
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
        let mut tx = pool.begin().await?;

        assert_eq!(
            super::super::get_path_by_directory(
                &mut tx,
                &LibraryDirectoryPath::from_str("test/d%i_r$")?
            )
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
        let mut tx = pool.begin().await?;

        // フォルダID 11 には2曲存在するため true
        assert!(super::super::is_exist_in_folder(&mut tx, 11).await?);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_is_exist_in_folder"))]
    async fn フォルダに1曲だけ存在する場合(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        // フォルダID 22 には1曲存在するため true
        assert!(super::super::is_exist_in_folder(&mut tx, 22).await?);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("test_is_exist_in_folder"))]
    async fn フォルダに曲が存在しない場合(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        // フォルダID 99 には曲が存在しないため false
        assert!(!super::super::is_exist_in_folder(&mut tx, 99).await?);

        Ok(())
    }
}
