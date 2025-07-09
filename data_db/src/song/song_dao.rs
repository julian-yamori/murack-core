use super::{SongEntry, SongRow};
use crate::{
    converts::{DbFolderIdMayRoot, DbLibSongPath, DbLibSongPathRef},
    like_esc, sql_func,
};
use anyhow::Result;
use domain::{
    db_wrapper::TransactionWrapper,
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibSongPath},
};
use mockall::automock;
use rusqlite::{Row, named_params, params};

/// songテーブルのDAO
#[automock]
pub trait SongDao {
    /// IDを指定して1行取得
    fn select_by_id<'c>(&self, tx: &TransactionWrapper<'c>, id: i32) -> Result<Option<SongRow>>;

    /// パスを指定してrowidを取得
    fn select_id_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibSongPath,
    ) -> Result<Option<i32>>;

    /// 全レコードのパスを取得
    fn select_path_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<Vec<LibSongPath>>;

    /// 指定されたディレクトリで始まるパスを取得
    fn select_path_begins_directory<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>>;

    /// 全レコード数を取得
    fn count_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<u32>;

    /// 指定されたフォルダIDのレコード数を取得
    fn count_by_folder_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<u32>;

    /// 指定されたpathのレコードが存在するか確認
    fn exists_path<'c>(&self, tx: &TransactionWrapper<'c>, path: &LibSongPath) -> Result<bool>;

    /// 新規登録
    ///
    /// # Returns
    /// 登録されたレコードのrowid
    fn insert<'c, 'e>(&self, tx: &TransactionWrapper<'c>, entry: &SongEntry<'e>) -> Result<i32>;

    /// 旧パスを指定し、曲のパス情報を更新
    fn update_path_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()>;

    /// IDを指定し、再生時間を更新
    fn update_duration_by_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()>;

    /// 曲レコードを削除
    fn delete<'c>(&self, tx: &TransactionWrapper<'c>, song_id: i32) -> Result<()>;
}

/// SongDaoの本実装
pub struct SongDaoImpl {}

impl SongDao for SongDaoImpl {
    /// IDを指定して1行取得
    fn select_by_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
    ) -> Result<Option<SongRow>> {
        let sql = format!("select {} from [song] where [rowid] = ?", ALL_COLUMNS);
        sql_func::select_opt(tx, &sql, params![song_id], map_all)
    }

    /// パスを指定してrowidを取得
    fn select_id_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibSongPath,
    ) -> Result<Option<i32>> {
        sql_func::select_opt(
            tx,
            "select [rowid] from [song] where [path] = ?",
            params![DbLibSongPathRef::from(path)],
            |row| row.get(0),
        )
    }

    /// 全レコードのパスを取得
    fn select_path_all(&self, tx: &TransactionWrapper) -> Result<Vec<LibSongPath>> {
        sql_func::select_list(tx, "select [path] from [song]", [], |row| {
            let path: DbLibSongPath = row.get(0)?;
            Ok(path.into())
        })
    }

    /// 指定されたディレクトリで始まるパスを取得
    fn select_path_begins_directory(
        &self,
        tx: &TransactionWrapper,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>> {
        let path_str = path.as_str();

        //LIKE文エスケープ
        let cmp_value_buff;
        let (like_query, cmp_value) = if like_esc::is_need(path_str) {
            cmp_value_buff = like_esc::escape(path_str);
            ("like ? || '%' escape '$'", cmp_value_buff.as_str())
        } else {
            ("like ? || '%'", path_str)
        };

        let sql = format!("select [path] from [song] where [path] {}", like_query);
        sql_func::select_list(tx, &sql, params![cmp_value], |row| {
            let path: DbLibSongPath = row.get(0)?;
            Ok(path.into())
        })
    }

    /// 全レコード数を取得
    fn count_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<u32> {
        sql_func::select_val(tx, "select count(*) from [song]", [])
    }

    /// 指定されたフォルダIDのレコード数を取得
    fn count_by_folder_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<u32> {
        sql_func::select_val(
            tx,
            "select count(*) from [song] where [folder_id] is ?",
            params![DbFolderIdMayRoot::from(folder_id)],
        )
    }

    /// 指定されたpathのレコードが存在するか確認
    fn exists_path<'c>(&self, tx: &TransactionWrapper<'c>, path: &LibSongPath) -> Result<bool> {
        let count: u32 = sql_func::select_val(
            tx,
            "select count(*) from [song] where [path] = ?",
            params![DbLibSongPathRef::from(path)],
        )?;
        Ok(count > 0)
    }

    /// 新規登録
    ///
    /// # Returns
    /// 登録されたレコードのrowid
    fn insert<'c, 'e>(&self, tx: &TransactionWrapper<'c>, entry: &SongEntry<'e>) -> Result<i32> {
        let sql = "insert into [song]([duration],[path],[folder_id],[title],[artist],[album],[genre],[album_artist],[composer],[track_number],[track_max],[disc_number],[disc_max],[release_date],[rating],[original_song],[suggest_target],[memo],[memo_manage],[lyrics],[title_order],[artist_order],[album_order],[album_artist_order],[composer_order],[genre_order],[entry_date]) values(:duration,:path,:folder_id,:title,:artist,:album,:genre,:album_artist,:composer,:track_number,:track_max,:disc_number,:disc_max,:release_date,:rating,:original_song,:suggest_target,:memo,:memo_manage,:lyrics,:title_order,:artist_order,:album_order,:album_artist_order,:composer_order,:genre_order,:entry_date)";
        sql_func::insert_get(
            tx,
            sql,
            named_params! {
                ":duration": entry.duration,
                ":path": entry.path,
                ":folder_id": entry.folder_id,
                ":title": entry.title,
                ":artist": entry.artist,
                ":album": entry.album,
                ":genre": entry.genre,
                ":album_artist": entry.album_artist,
                ":composer": entry.composer,
                ":track_number": entry.track_number,
                ":track_max": entry.track_max,
                ":disc_number": entry.disc_number,
                ":disc_max": entry.disc_max,
                ":release_date": entry.release_date,
                ":rating": entry.rating,
                ":original_song": entry.original_song,
                ":suggest_target": entry.suggest_target,
                ":memo": entry.memo,
                ":memo_manage": entry.memo_manage,
                ":lyrics": entry.lyrics,
                ":title_order": entry.title_order,
                ":artist_order": entry.artist_order,
                ":album_order": entry.album_order,
                ":album_artist_order": entry.album_artist_order,
                ":composer_order": entry.composer_order,
                ":genre_order": entry.genre_order,
                ":entry_date": entry.entry_date,
            },
        )
    }

    /// 旧パスを指定し、曲のパス情報を更新
    fn update_path_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()> {
        sql_func::execute(
            tx,
            "update [song] set [path] = ?, [folder_id] = ? where [path] = ?",
            params![
                &DbLibSongPathRef::from(new_path),
                &DbFolderIdMayRoot::from(new_folder_id),
                &DbLibSongPathRef::from(old_path),
            ],
        )
    }

    /// IDを指定し、再生時間を更新
    fn update_duration_by_id<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()> {
        sql_func::execute(
            tx,
            "update [song] set [duration] = ? where [rowid] = ?",
            params![duration, song_id],
        )
    }

    /// 曲レコードを削除
    fn delete<'c>(&self, tx: &TransactionWrapper<'c>, song_id: i32) -> Result<()> {
        sql_func::execute(tx, "delete from [song] where [rowid] = ?", params![song_id])
    }
}

/// 全カラム名
const ALL_COLUMNS: &str = "[rowid],[duration],[path],[folder_id],[title],[artist],[album],[genre],[album_artist],[composer],[track_number],[track_max],[disc_number],[disc_max],[release_date],[rating],[original_song],[suggest_target],[memo],[memo_manage],[lyrics],[title_order],[artist_order],[album_order],[album_artist_order],[composer_order],[genre_order],[entry_date]";

/// 全カラム取得時のマッパー
///
/// 実用時によく使うのはSongSyncのはずなので、
/// こちらは速度よりもメンテしやすさを重視してカラム名指定。
fn map_all(row: &Row) -> rusqlite::Result<SongRow> {
    Ok(SongRow {
        rowid: row.get("rowid")?,
        duration: row.get("duration")?,
        path: row.get("path")?,
        folder_id: row.get("folder_id")?,
        title: row.get("title")?,
        artist: row.get("artist")?,
        album: row.get("album")?,
        genre: row.get("genre")?,
        album_artist: row.get("album_artist")?,
        composer: row.get("composer")?,
        track_number: row.get("track_number")?,
        track_max: row.get("track_max")?,
        disc_number: row.get("disc_number")?,
        disc_max: row.get("disc_max")?,
        release_date: row.get("release_date")?,
        memo: row.get("memo")?,
        rating: row.get("rating")?,
        original_song: row.get("original_song")?,
        suggest_target: row.get("suggest_target")?,
        memo_manage: row.get("memo_manage")?,
        lyrics: row.get("lyrics")?,
        title_order: row.get("title_order")?,
        artist_order: row.get("artist_order")?,
        album_order: row.get("album_order")?,
        genre_order: row.get("genre_order")?,
        album_artist_order: row.get("album_artist_order")?,
        composer_order: row.get("composer_order")?,
        entry_date: row.get("entry_date")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initialize;
    use chrono::NaiveDate;
    use domain::db_wrapper::ConnectionFactory;

    fn entry_fill(song_path: &LibSongPath) -> SongEntry {
        SongEntry {
            duration: 123,
            path: song_path.into(),
            folder_id: FolderIdMayRoot::Folder(34).into(),
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
            release_date: Some(NaiveDate::from_ymd(1998, 8, 31).into()),
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
            entry_date: NaiveDate::from_ymd(2021, 9, 20).into(),
        }
    }
    fn entry_empty(song_path: &LibSongPath) -> SongEntry {
        SongEntry {
            duration: 0,
            path: song_path.into(),
            folder_id: FolderIdMayRoot::Root.into(),
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
            entry_date: NaiveDate::from_ymd(2021, 9, 20).into(),
        }
    }

    #[test]
    fn test_insert_select_fill() {
        let song_path = LibSongPath::new("test/hoge.flac");
        let entry = entry_fill(&song_path);

        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};
        let rowid = target.insert(&tx, &entry).unwrap();
        let row = target.select_by_id(&tx, rowid).unwrap().unwrap();

        assert_eq!(row.rowid, rowid);
        entry.assert_eq_row(&row);
    }

    #[test]
    fn test_insert_select_empty() {
        let song_path = LibSongPath::new("fuga.flac");
        let entry = entry_empty(&song_path);

        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};
        let rowid = target.insert(&tx, &entry).unwrap();
        let row = target.select_by_id(&tx, rowid).unwrap().unwrap();

        assert_eq!(row.rowid, rowid);
        entry.assert_eq_row(&row);
    }

    #[test]
    fn test_select_id_by_path() {
        let song_path = LibSongPath::new("test/hoge.flac");

        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};
        let rowid = target.insert(&tx, &entry_fill(&song_path)).unwrap();

        assert_eq!(
            target.select_id_by_path(&tx, &song_path).unwrap().unwrap(),
            rowid
        );
    }

    #[test]
    fn test_select_path_all() {
        let song_paths = vec![
            LibSongPath::new("test/hoge.flac"),
            LibSongPath::new("fuga.flac"),
        ];

        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};
        target.insert(&tx, &entry_fill(&song_paths[0])).unwrap();
        target.insert(&tx, &entry_fill(&song_paths[1])).unwrap();

        assert_eq!(target.select_path_all(&tx).unwrap(), song_paths);
    }

    #[test]
    fn test_select_path_begins_directory() {
        let song_paths = vec![
            LibSongPath::new("test/hoge.flac"),
            LibSongPath::new("test/hoge2.flac"),
            LibSongPath::new("fuga.flac"),
            LibSongPath::new("dummy/fuga.flac"),
            LibSongPath::new("test/dir/hoge3.flac"),
            LibSongPath::new("dummy/test/dir/dummy.mp3"),
        ];

        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};
        for path in &song_paths {
            target.insert(&tx, &entry_fill(path)).unwrap();
        }

        assert_eq!(
            target
                .select_path_begins_directory(&tx, &LibDirPath::new("test"))
                .unwrap(),
            vec![
                LibSongPath::new("test/hoge.flac"),
                LibSongPath::new("test/hoge2.flac"),
                LibSongPath::new("test/dir/hoge3.flac"),
            ]
        );
        assert_eq!(
            target
                .select_path_begins_directory(&tx, &LibDirPath::new("test/dir"))
                .unwrap(),
            vec![LibSongPath::new("test/dir/hoge3.flac"),]
        );
        assert_eq!(
            target
                .select_path_begins_directory(&tx, &LibDirPath::new(""))
                .unwrap(),
            song_paths
        );
    }
    #[test]
    fn test_select_path_begins_directory_esc() {
        let song_paths = vec![
            LibSongPath::new("test/d%i_r$/hoge.flac"),
            LibSongPath::new("test/dZi_r$/dummy.flac"),
            LibSongPath::new("fuga.flac"),
            LibSongPath::new("dummy/test/d%i_r$/dummy.mp3"),
        ];

        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};
        for path in song_paths {
            target.insert(&tx, &entry_fill(&path)).unwrap();
        }

        assert_eq!(
            target
                .select_path_begins_directory(&tx, &LibDirPath::new("test/d%i_r$"))
                .unwrap(),
            vec![LibSongPath::new("test/d%i_r$/hoge.flac"),]
        );
    }

    #[test]
    fn test_count_all_3() {
        let song_paths = vec![
            LibSongPath::new("test/hoge.flac"),
            LibSongPath::new("fuga.flac"),
            LibSongPath::new("piyo.mp3"),
        ];

        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};
        target.insert(&tx, &entry_fill(&song_paths[0])).unwrap();
        target.insert(&tx, &entry_fill(&song_paths[1])).unwrap();
        target.insert(&tx, &entry_empty(&song_paths[2])).unwrap();

        assert_eq!(target.count_all(&tx).unwrap(), 3);
    }
    #[test]
    fn test_count_all_none() {
        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};

        assert_eq!(target.count_all(&tx).unwrap(), 0);
    }

    #[test]
    fn test_count_by_folder_id() {
        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};

        let path = LibSongPath::new("test/hoge.flac");
        let mut entry = entry_fill(&path);
        entry.folder_id = FolderIdMayRoot::Folder(11).into();
        target.insert(&tx, &entry).unwrap();

        let path = LibSongPath::new("dummy/fuga.flac");
        let mut entry = entry_fill(&path);
        entry.folder_id = FolderIdMayRoot::Folder(22).into();
        target.insert(&tx, &entry).unwrap();

        let path = LibSongPath::new("test/piyo.flac");
        let mut entry = entry_fill(&path);
        entry.folder_id = FolderIdMayRoot::Folder(11).into();
        target.insert(&tx, &entry).unwrap();

        let path = LibSongPath::new("piyo.flac");
        let mut entry = entry_fill(&path);
        entry.folder_id = FolderIdMayRoot::Root.into();
        target.insert(&tx, &entry).unwrap();

        assert_eq!(
            target
                .count_by_folder_id(&tx, FolderIdMayRoot::Folder(11))
                .unwrap(),
            2
        );
        assert_eq!(
            target
                .count_by_folder_id(&tx, FolderIdMayRoot::Root)
                .unwrap(),
            1
        );
    }

    #[test]
    fn test_exists_path() {
        let song_paths = vec![
            LibSongPath::new("test/hoge.flac"),
            LibSongPath::new("fuga.flac"),
            LibSongPath::new("piyo.mp3"),
        ];

        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};
        target.insert(&tx, &entry_fill(&song_paths[0])).unwrap();
        target.insert(&tx, &entry_fill(&song_paths[1])).unwrap();
        target.insert(&tx, &entry_empty(&song_paths[2])).unwrap();

        assert!(
            target
                .exists_path(&tx, &LibSongPath::new("test/hoge.flac"))
                .unwrap()
        );
        assert!(
            !target
                .exists_path(&tx, &LibSongPath::new("none.m4a"))
                .unwrap()
        );
    }

    #[test]
    fn test_delete() {
        let song_paths = vec![
            LibSongPath::new("test/hoge.flac"),
            LibSongPath::new("fuga.flac"),
            LibSongPath::new("piyo.mp3"),
        ];

        let mut db = ConnectionFactory::Memory.open().unwrap();
        let tx = db.transaction().unwrap();
        initialize::song(&tx).unwrap();

        let target = SongDaoImpl {};
        target.insert(&tx, &entry_fill(&song_paths[0])).unwrap();
        let id1 = target.insert(&tx, &entry_fill(&song_paths[1])).unwrap();
        target.insert(&tx, &entry_empty(&song_paths[2])).unwrap();

        target.delete(&tx, id1).unwrap();

        assert_eq!(target.select_by_id(&tx, id1).unwrap(), None);
        assert_eq!(target.count_all(&tx).unwrap(), 2);
    }
}
