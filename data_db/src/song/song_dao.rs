use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;
use murack_core_domain::{
    db::DbTransaction,
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibSongPath},
};
use sqlx::{Row, postgres::PgRow};

use super::{SongEntry, SongRow};
use crate::{converts::enums::db_from_folder_id_may_root, like_esc};

/// songテーブルのDAO
#[async_trait]
pub trait SongDao {
    /// IDを指定して1行取得
    async fn select_by_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        id: i32,
    ) -> Result<Option<SongRow>>;

    /// パスを指定してrowidを取得
    async fn select_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<i32>>;

    /// 全レコードのパスを取得
    async fn select_path_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<Vec<LibSongPath>>;

    /// 指定されたディレクトリで始まるパスを取得
    async fn select_path_begins_directory<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>>;

    /// 全レコード数を取得
    async fn count_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<u32>;

    /// 指定されたフォルダIDのレコード数を取得
    async fn count_by_folder_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<u32>;

    /// 指定されたpathのレコードが存在するか確認
    async fn exists_path<'c>(&self, tx: &mut DbTransaction<'c>, path: &LibSongPath)
    -> Result<bool>;

    /// 新規登録
    ///
    /// # Returns
    /// 登録されたレコードのrowid
    async fn insert<'c, 'e>(
        &self,
        tx: &mut DbTransaction<'c>,
        entry: &SongEntry<'e>,
    ) -> Result<i32>;

    /// 旧パスを指定し、曲のパス情報を更新
    async fn update_path_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()>;

    /// IDを指定し、再生時間を更新
    async fn update_duration_by_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()>;

    /// 曲レコードを削除
    async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, song_id: i32) -> Result<()>;
}

/// SongDaoの本実装
pub struct SongDaoImpl {}

#[async_trait]
impl SongDao for SongDaoImpl {
    /// IDを指定して1行取得
    async fn select_by_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<Option<SongRow>> {
        let row = sqlx::query_as!(SongRow, "SELECT id, duration, path, folder_id, title, artist, album, genre, album_artist, composer, track_number, track_max, disc_number, disc_max, release_date, rating, original_track, suggest_target, memo,memo_manage, lyrics, title_order, artist_order, album_order, album_artist_order, composer_order, genre_order, created_at FROM tracks WHERE id = $1", song_id).fetch_optional(&mut **tx.get()).await?;

        Ok(row)
    }

    /// パスを指定してrowidを取得
    async fn select_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<i32>> {
        let id = sqlx::query_scalar!("SELECT id FROM tracks WHERE path = $1", path.as_str(),)
            .fetch_optional(&mut **tx.get())
            .await?;

        Ok(id)
    }

    /// 全レコードのパスを取得
    async fn select_path_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<Vec<LibSongPath>> {
        let paths = sqlx::query!("SELECT path FROM tracks",)
            .map(|row| LibSongPath::new(row.path))
            .fetch_all(&mut **tx.get())
            .await?;

        Ok(paths)
    }

    /// 指定されたディレクトリで始まるパスを取得
    async fn select_path_begins_directory<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>> {
        let path_str = path.as_str();

        //LIKE文エスケープ
        let cmp_value_buff;
        let (like_query, cmp_value) = if like_esc::is_need(path_str) {
            cmp_value_buff = like_esc::escape(path_str);
            ("LIKE $1 || '%' ESCAPE '$'", cmp_value_buff.as_str())
        } else {
            ("LIKE $1 || '%'", path_str)
        };

        let sql = format!("SELECT path FROM tracks WHERE path {like_query}");
        let paths = sqlx::query(&sql)
            .bind(cmp_value)
            .map(|row: PgRow| LibSongPath::new(row.get::<&str, _>(0)))
            .fetch_all(&mut **tx.get())
            .await?;

        Ok(paths)
    }

    /// 全レコード数を取得
    async fn count_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<u32> {
        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM tracks"#)
            .fetch_one(&mut **tx.get())
            .await?;

        Ok(count.try_into()?)
    }

    /// 指定されたフォルダIDのレコード数を取得
    async fn count_by_folder_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<u32> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE folder_id IS NOT DISTINCT FROM $1"#,
            db_from_folder_id_may_root(folder_id),
        )
        .fetch_one(&mut **tx.get())
        .await?;

        Ok(count.try_into()?)
    }

    /// 指定されたpathのレコードが存在するか確認
    async fn exists_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<bool> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE path = $1"#,
            path.as_str(),
        )
        .fetch_one(&mut **tx.get())
        .await?;
        Ok(count > 0)
    }

    /// 新規登録
    ///
    /// # Returns
    /// 登録されたレコードのrowid
    async fn insert<'c, 'e>(
        &self,
        tx: &mut DbTransaction<'c>,
        entry: &SongEntry<'e>,
    ) -> Result<i32> {
        let id = sqlx::query_scalar!(
            "INSERT INTO tracks (duration, path, folder_id, title, artist, album, genre, album_artist, composer, track_number, track_max, disc_number, disc_max, release_date, rating, original_track,suggest_target, memo, memo_manage, lyrics, title_order, artist_order, album_order, album_artist_order, composer_order, genre_order) values($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26) RETURNING id",
                entry.duration,
                entry.path,
                entry.folder_id,
                entry.title,
                entry.artist,
                entry.album,
                entry.genre,
                entry.album_artist,
                entry.composer,
                entry.track_number,
                entry.track_max,
                entry.disc_number,
                entry.disc_max,
                entry.release_date,
                entry.rating,
                entry.original_song,
                entry.suggest_target,
                entry.memo,
                entry.memo_manage,
                entry.lyrics,
                entry.title_order,
                entry.artist_order,
                entry.album_order,
                entry.album_artist_order,
                entry.composer_order,
                entry.genre_order,
        ).fetch_one(&mut **tx.get()).await?;

        Ok(id)
    }

    /// 旧パスを指定し、曲のパス情報を更新
    async fn update_path_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE tracks SET path = $1, folder_id = $2 WHERE path = $3",
            new_path.as_str(),
            db_from_folder_id_may_root(new_folder_id),
            old_path.as_str(),
        )
        .execute(&mut **tx.get())
        .await?;

        Ok(())
    }

    /// IDを指定し、再生時間を更新
    async fn update_duration_by_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE tracks SET duration = $1 WHERE id = $2",
            i32::try_from(duration)?,
            song_id,
        )
        .execute(&mut **tx.get())
        .await?;

        Ok(())
    }

    /// 曲レコードを削除
    async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, song_id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM tracks WHERE id = $1", song_id)
            .execute(&mut **tx.get())
            .await?;

        Ok(())
    }
}

#[derive(Default)]
pub struct MockSongDao {
    pub inner: MockSongDaoInner,
}
#[async_trait]
impl SongDao for MockSongDao {
    async fn select_by_id<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        id: i32,
    ) -> Result<Option<SongRow>> {
        self.inner.select_by_id(id)
    }

    async fn select_id_by_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<i32>> {
        self.inner.select_id_by_path(path)
    }

    async fn select_path_all<'c>(&self, _db: &mut DbTransaction<'c>) -> Result<Vec<LibSongPath>> {
        self.inner.select_path_all()
    }

    async fn select_path_begins_directory<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>> {
        self.inner.select_path_begins_directory(path)
    }

    async fn count_all<'c>(&self, _db: &mut DbTransaction<'c>) -> Result<u32> {
        self.inner.count_all()
    }

    async fn count_by_folder_id<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<u32> {
        self.inner.count_by_folder_id(folder_id)
    }

    async fn exists_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<bool> {
        self.inner.exists_path(path)
    }

    async fn insert<'c, 'e>(
        &self,
        _db: &mut DbTransaction<'c>,
        entry: &SongEntry<'e>,
    ) -> Result<i32> {
        self.inner.insert(entry)
    }

    async fn update_path_by_path<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()> {
        self.inner
            .update_path_by_path(old_path, new_path, new_folder_id)
    }

    async fn update_duration_by_id<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()> {
        self.inner.update_duration_by_id(song_id, duration)
    }

    async fn delete<'c>(&self, _db: &mut DbTransaction<'c>, song_id: i32) -> Result<()> {
        self.inner.delete(song_id)
    }
}
mock! {
    pub SongDaoInner {
        pub fn select_by_id(&self, id: i32) -> Result<Option<SongRow>>;

        pub fn select_id_by_path(
            &self,
            path: &LibSongPath,
        ) -> Result<Option<i32>>;

        pub fn select_path_all(&self) -> Result<Vec<LibSongPath>>;

        pub fn select_path_begins_directory(
            &self,
            path: &LibDirPath,
        ) -> Result<Vec<LibSongPath>>;

        pub fn count_all(&self) -> Result<u32>;

        pub fn count_by_folder_id(
            &self,
            folder_id: FolderIdMayRoot,
        ) -> Result<u32>;

        pub fn exists_path(&self, path: &LibSongPath) -> Result<bool>;

        pub fn insert <'e>(&self, entry: &SongEntry<'e>) -> Result<i32>;

        pub fn update_path_by_path(
            &self,
            old_path: &LibSongPath,
            new_path: &LibSongPath,
            new_folder_id: FolderIdMayRoot,
        ) -> Result<()>;

        pub fn update_duration_by_id(
            &self,
            song_id: i32,
            duration: u32,
        ) -> Result<()>;

        pub fn delete(&self, song_id: i32) -> Result<()>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;
    use sqlx::PgPool;

    fn entry_fill(song_path: &LibSongPath) -> SongEntry {
        SongEntry {
            duration: 123,
            path: song_path.as_str(),
            folder_id: Some(34),
            title: "曲名",
            artist: "アーティスト",
            album: "The Album",
            genre: "ジャンル",
            album_artist: "ｱﾙﾊﾞﾑｱｰﾃｨｽﾄ",
            composer: "作曲者",
            track_number: Some(12),
            track_max: Some(34),
            disc_number: Some(56),
            disc_max: Some(789),
            release_date: Some(NaiveDate::from_ymd_opt(1998, 8, 31).unwrap()),
            rating: 5,
            original_song: "原曲",
            suggest_target: true,
            memo: "メモ",
            memo_manage: "管理用のメモ\nメモ",
            lyrics: "1行目\n2nd row\n\n次の節\n",
            title_order: "曲名",
            artist_order: "あーてぃすと",
            album_order: "the album",
            album_artist_order: "あるばむあーてぃすと",
            composer_order: "さっきょくしゃ",
            genre_order: "じゃんる",
        }
    }
    fn entry_empty(song_path: &LibSongPath) -> SongEntry {
        SongEntry {
            duration: 0,
            path: song_path.as_str(),
            folder_id: None,
            title: "",
            artist: "",
            album: "",
            genre: "",
            album_artist: "",
            composer: "",
            track_number: None,
            track_max: None,
            disc_number: None,
            disc_max: None,
            release_date: None,
            rating: 0,
            original_song: "",
            suggest_target: false,
            memo: "",
            memo_manage: "",
            lyrics: "",
            title_order: "",
            artist_order: "",
            album_order: "",
            album_artist_order: "",
            composer_order: "",
            genre_order: "",
        }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_insert_select_fill(db_pool: PgPool) -> anyhow::Result<()> {
        let song_path = LibSongPath::new("test/hoge.flac");
        let entry = entry_fill(&song_path);

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};
        let rowid = target.insert(&mut tx, &entry).await?;
        let row = target.select_by_id(&mut tx, rowid).await?.unwrap();

        assert_eq!(row.id, rowid);
        entry.assert_eq_row(&row);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_insert_select_empty(db_pool: PgPool) -> anyhow::Result<()> {
        let song_path = LibSongPath::new("fuga.flac");
        let entry = entry_empty(&song_path);

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};
        let rowid = target.insert(&mut tx, &entry).await?;
        let row = target.select_by_id(&mut tx, rowid).await?.unwrap();

        assert_eq!(row.id, rowid);
        entry.assert_eq_row(&row);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_select_id_by_path(db_pool: PgPool) -> anyhow::Result<()> {
        let song_path = LibSongPath::new("test/hoge.flac");

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};
        let rowid = target.insert(&mut tx, &entry_fill(&song_path)).await?;

        assert_eq!(
            target
                .select_id_by_path(&mut tx, &song_path)
                .await?
                .unwrap(),
            rowid
        );

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_select_path_all(db_pool: PgPool) -> anyhow::Result<()> {
        let song_paths = vec![
            LibSongPath::new("test/hoge.flac"),
            LibSongPath::new("fuga.flac"),
        ];

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};
        target.insert(&mut tx, &entry_fill(&song_paths[0])).await?;
        target.insert(&mut tx, &entry_fill(&song_paths[1])).await?;

        assert_eq!(target.select_path_all(&mut tx).await?, song_paths);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_select_path_begins_directory(db_pool: PgPool) -> anyhow::Result<()> {
        let song_paths = vec![
            LibSongPath::new("test/hoge.flac"),
            LibSongPath::new("test/hoge2.flac"),
            LibSongPath::new("fuga.flac"),
            LibSongPath::new("dummy/fuga.flac"),
            LibSongPath::new("test/dir/hoge3.flac"),
            LibSongPath::new("dummy/test/dir/dummy.mp3"),
        ];

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};
        for path in &song_paths {
            target.insert(&mut tx, &entry_fill(path)).await?;
        }

        assert_eq!(
            target
                .select_path_begins_directory(&mut tx, &LibDirPath::new("test"))
                .await?,
            vec![
                LibSongPath::new("test/hoge.flac"),
                LibSongPath::new("test/hoge2.flac"),
                LibSongPath::new("test/dir/hoge3.flac"),
            ]
        );
        assert_eq!(
            target
                .select_path_begins_directory(&mut tx, &LibDirPath::new("test/dir"))
                .await?,
            vec![LibSongPath::new("test/dir/hoge3.flac"),]
        );
        assert_eq!(
            target
                .select_path_begins_directory(&mut tx, &LibDirPath::new(""))
                .await?,
            song_paths
        );

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_select_path_begins_directory_esc(db_pool: PgPool) -> anyhow::Result<()> {
        let song_paths = vec![
            LibSongPath::new("test/d%i_r$/hoge.flac"),
            LibSongPath::new("test/dZi_r$/dummy.flac"),
            LibSongPath::new("fuga.flac"),
            LibSongPath::new("dummy/test/d%i_r$/dummy.mp3"),
        ];

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};
        for path in song_paths {
            target.insert(&mut tx, &entry_fill(&path)).await?;
        }

        assert_eq!(
            target
                .select_path_begins_directory(&mut tx, &LibDirPath::new("test/d%i_r$"))
                .await?,
            vec![LibSongPath::new("test/d%i_r$/hoge.flac"),]
        );

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_count_all_3(db_pool: PgPool) -> anyhow::Result<()> {
        let song_paths = [
            LibSongPath::new("test/hoge.flac"),
            LibSongPath::new("fuga.flac"),
            LibSongPath::new("piyo.mp3"),
        ];

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};
        target.insert(&mut tx, &entry_fill(&song_paths[0])).await?;
        target.insert(&mut tx, &entry_fill(&song_paths[1])).await?;
        target.insert(&mut tx, &entry_empty(&song_paths[2])).await?;

        assert_eq!(target.count_all(&mut tx).await?, 3);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_count_all_none(db_pool: PgPool) -> anyhow::Result<()> {
        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};

        assert_eq!(target.count_all(&mut tx).await?, 0);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_count_by_folder_id(db_pool: PgPool) -> anyhow::Result<()> {
        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};

        let path = LibSongPath::new("test/hoge.flac");
        let mut entry = entry_fill(&path);
        entry.folder_id = Some(11);
        target.insert(&mut tx, &entry).await?;

        let path = LibSongPath::new("dummy/fuga.flac");
        let mut entry = entry_fill(&path);
        entry.folder_id = Some(22);
        target.insert(&mut tx, &entry).await?;

        let path = LibSongPath::new("test/piyo.flac");
        let mut entry = entry_fill(&path);
        entry.folder_id = Some(11);
        target.insert(&mut tx, &entry).await?;

        let path = LibSongPath::new("piyo.flac");
        let mut entry = entry_fill(&path);
        entry.folder_id = None;
        target.insert(&mut tx, &entry).await?;

        assert_eq!(
            target
                .count_by_folder_id(&mut tx, FolderIdMayRoot::Folder(11))
                .await?,
            2
        );
        assert_eq!(
            target
                .count_by_folder_id(&mut tx, FolderIdMayRoot::Root)
                .await?,
            1
        );

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_exists_path(db_pool: PgPool) -> anyhow::Result<()> {
        let song_paths = [
            LibSongPath::new("test/hoge.flac"),
            LibSongPath::new("fuga.flac"),
            LibSongPath::new("piyo.mp3"),
        ];

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};
        target.insert(&mut tx, &entry_fill(&song_paths[0])).await?;
        target.insert(&mut tx, &entry_fill(&song_paths[1])).await?;
        target.insert(&mut tx, &entry_empty(&song_paths[2])).await?;

        assert!(
            target
                .exists_path(&mut tx, &LibSongPath::new("test/hoge.flac"))
                .await?
        );
        assert!(
            !target
                .exists_path(&mut tx, &LibSongPath::new("none.m4a"))
                .await?
        );

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_delete(db_pool: PgPool) -> anyhow::Result<()> {
        let song_paths = [
            LibSongPath::new("test/hoge.flac"),
            LibSongPath::new("fuga.flac"),
            LibSongPath::new("piyo.mp3"),
        ];

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        let target = SongDaoImpl {};
        target.insert(&mut tx, &entry_fill(&song_paths[0])).await?;
        let id1 = target.insert(&mut tx, &entry_fill(&song_paths[1])).await?;
        target.insert(&mut tx, &entry_empty(&song_paths[2])).await?;

        target.delete(&mut tx, id1).await?;

        assert_eq!(target.select_by_id(&mut tx, id1).await?, None);
        assert_eq!(target.count_all(&mut tx).await?, 2);

        Ok(())
    }
}
