use anyhow::Result;
use murack_core_domain::{
    db::DbTransaction,
    path::{LibDirPath, LibPathStr, LibTrackPath},
    track::DbTrackRepository,
};
use sqlx::PgPool;

use crate::track::DbTrackRepositoryImpl;

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

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::from("test/hoge".to_owned()))
            .await?;

        // 結果は3つの楽曲パスであるはず
        assert_eq!(result.len(), 3);
        assert!(result.contains(&LibTrackPath::new("test/hoge/track1.mp3")));
        assert!(result.contains(&LibTrackPath::new("test/hoge/track2.flac")));
        assert!(result.contains(&LibTrackPath::new("test/hoge/track3.m4a")));

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

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::from("test/hoge".to_owned()))
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

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::from("test/hoge.flac".to_owned()))
            .await?;

        // 結果は指定した楽曲ファイル1つであるはず
        assert_eq!(result, vec![LibTrackPath::new("test/hoge.flac")]);

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
        let target = DbTrackRepositoryImpl::new();

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::root())
            .await?;

        // 結果は3つの楽曲パスであるはず
        assert_eq!(result.len(), 3);
        assert!(result.contains(&LibTrackPath::new("track1.mp3")));
        assert!(result.contains(&LibTrackPath::new("test/hoge/track2.flac")));
        assert!(result.contains(&LibTrackPath::new("test/track3.m4a")));

        // データベース内の全楽曲数を確認
        let total_count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM tracks"#)
            .fetch_one(&mut **tx.get())
            .await?;
        assert_eq!(total_count, 3);

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

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        assert_eq!(
            target
                .get_path_by_directory(&mut tx, &LibDirPath::new("test"))
                .await?,
            vec![
                LibTrackPath::new("test/hoge.flac"),
                LibTrackPath::new("test/hoge2.flac"),
                LibTrackPath::new("test/dir/hoge3.flac"),
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

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        assert_eq!(
            target
                .get_path_by_directory(&mut tx, &LibDirPath::new("test/dir"))
                .await?,
            vec![LibTrackPath::new("test/dir/hoge3.flac"),]
        );

        Ok(())
    }

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/test_get_path_by_directory/normal_chars.sql")
    )]
    fn ルート指定(pool: PgPool) -> anyhow::Result<()> {
        let target = DbTrackRepositoryImpl::new();

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        assert_eq!(
            target
                .get_path_by_directory(&mut tx, &LibDirPath::root())
                .await?,
            vec![
                LibTrackPath::new("test/hoge.flac"),
                LibTrackPath::new("test/hoge2.flac"),
                LibTrackPath::new("fuga.flac"),
                LibTrackPath::new("dummy/fuga.flac"),
                LibTrackPath::new("test/dir/hoge3.flac"),
                LibTrackPath::new("dummy/test/dir/dummy.mp3"),
            ]
        );

        Ok(())
    }

    #[sqlx::test(
        migrator = "crate::MIGRATOR",
        fixtures("fixtures/test_get_path_by_directory/special_chars.sql")
    )]
    fn 特殊文字を挟むパスでの検索(pool: PgPool) -> anyhow::Result<()> {
        let target = DbTrackRepositoryImpl::new();

        let mut tx = DbTransaction::PgTransaction {
            tx: pool.begin().await?,
        };

        assert_eq!(
            target
                .get_path_by_directory(&mut tx, &LibDirPath::new("test/d%i_r$"))
                .await?,
            vec![LibTrackPath::new("test/d%i_r$/hoge.flac"),]
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
