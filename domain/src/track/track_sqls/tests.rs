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
