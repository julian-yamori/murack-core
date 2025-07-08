use super::SongDao;
use anyhow::Result;
use domain::{
    db_wrapper::TransactionWrapper,
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibPathStr, LibSongPath},
    song::DbSongRepository,
};
use std::rc::Rc;

/// HasDbSongRepositoryの本実装
#[derive(new)]
pub struct DbSongRepositoryImpl {
    song_dao: Rc<dyn SongDao>,
}

impl DbSongRepository for DbSongRepositoryImpl {
    /// パスから曲IDを取得
    fn get_id_by_path(&self, tx: &TransactionWrapper, path: &LibSongPath) -> Result<Option<i32>> {
        self.song_dao.select_id_by_path(tx, path)
    }

    /// 文字列でパスを指定して、該当曲のパスリストを取得
    fn get_path_by_path_str(
        &self,
        tx: &TransactionWrapper,
        path: &LibPathStr,
    ) -> Result<Vec<LibSongPath>> {
        //ディレクトリ指定とみなして検索
        let dir_path = path.to_dir_path();
        let mut list = self.get_path_by_directory(tx, &dir_path)?;

        //ファイル指定とみなしての検索でヒットしたら追加
        let song_path = path.to_song_path();
        if self.is_exist_path(tx, &song_path)? {
            list.push(song_path);
        }

        Ok(list)
    }

    /// ディレクトリを指定してパスを取得
    /// # Arguments
    /// - path: 検索対象のライブラリパス
    /// # Returns
    /// 指定されたディレクトリ内の、全ての曲のパス
    fn get_path_by_directory(
        &self,
        tx: &TransactionWrapper,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>> {
        //ルートフォルダ指定なら、全曲
        if path.is_root() {
            self.song_dao.select_path_all(tx)
        } else {
            self.song_dao.select_path_begins_directory(tx, path)
        }
    }

    /// ライブラリ内の全ての曲のパスを取得
    fn get_path_all(&self, tx: &TransactionWrapper) -> Result<Vec<LibSongPath>> {
        self.song_dao.select_path_all(tx)
    }

    /// 指定したパスの曲が存在するか確認
    fn is_exist_path(&self, tx: &TransactionWrapper, path: &LibSongPath) -> Result<bool> {
        self.song_dao.exists_path(tx, path)
    }

    /// 指定されたフォルダに曲が存在するか確認
    fn is_exist_in_folder(&self, tx: &TransactionWrapper, folder_id: i32) -> Result<bool> {
        let song_count = self
            .song_dao
            .count_by_folder_id(tx, FolderIdMayRoot::Folder(folder_id))?;
        Ok(song_count > 0)
    }

    /// 曲のパスを書き換え
    ///
    /// # Arguments
    /// - old_path: 書き換え元の曲のパス
    /// - new_path: 書き換え先の曲のパス
    /// - new_folder_id: 新しい親フォルダのID
    fn update_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()> {
        self.song_dao
            .update_path_by_path(tx, old_path, new_path, new_folder_id)
    }

    /// 曲の再生時間を書き換え
    fn update_duration<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()> {
        self.song_dao.update_duration_by_id(tx, song_id, duration)
    }

    /// 曲を削除
    ///
    /// # Arguments
    /// - song_id: 削除する曲のID
    fn delete(&self, tx: &TransactionWrapper, song_id: i32) -> Result<()> {
        self.song_dao.delete(tx, song_id)
    }
}

#[cfg(test)]
mod tests {
    use super::super::MockSongDao;
    use super::*;
    use domain::{db_wrapper::ConnectionFactory, mocks};
    use paste::paste;

    mocks! {DbSongRepositoryImpl, [
        SongDao
    ]}

    #[test]
    fn test_get_path_by_path_str_directory() {
        const DIR_PATH_STR: &str = "test/hoge";
        fn expect() -> Vec<LibSongPath> {
            vec![
                LibSongPath::new("test/hoge/song1.mp3"),
                LibSongPath::new("test/hoge/song2.flac"),
                LibSongPath::new("test/hoge/song3.m4a"),
            ]
        }

        let mut mocks = Mocks::new();
        mocks.song_dao(|m| {
            m.expect_select_path_begins_directory()
                .withf(|_, a_path| a_path == &LibDirPath::new(DIR_PATH_STR))
                .times(1)
                .returning(|_, _| Ok(expect()));
            m.expect_exists_path()
                .withf(|_, a_path| a_path == &LibSongPath::new(DIR_PATH_STR))
                .times(1)
                .returning(|_, _| Ok(false));
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let result = t
                .get_path_by_path_str(&tx, &LibPathStr::from(DIR_PATH_STR.to_owned()))
                .unwrap();
            assert_eq!(result, expect());
        });
    }

    #[test]
    fn test_get_path_by_path_str_not_found() {
        const DIR_PATH_STR: &str = "test/hoge";

        let mut mocks = Mocks::new();
        mocks.song_dao(|m| {
            m.expect_select_path_begins_directory()
                .withf(|_, a_path| a_path == &LibDirPath::new(DIR_PATH_STR))
                .times(1)
                .returning(|_, _| Ok(vec![]));
            m.expect_exists_path()
                .withf(|_, a_path| a_path == &LibSongPath::new(DIR_PATH_STR))
                .times(1)
                .returning(|_, _| Ok(false));
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let result = t
                .get_path_by_path_str(&tx, &LibPathStr::from(DIR_PATH_STR.to_owned()))
                .unwrap();
            assert_eq!(result, vec![]);
        });
    }

    #[test]
    fn test_get_path_by_path_str_song() {
        const DIR_PATH_STR: &str = "test/hoge.flac";

        let mut mocks = Mocks::new();
        mocks.song_dao(|m| {
            m.expect_select_path_begins_directory()
                .withf(|_, a_path| a_path == &LibDirPath::new(DIR_PATH_STR))
                .times(1)
                .returning(|_, _| Ok(vec![]));
            m.expect_exists_path()
                .withf(|_, a_path| a_path == &LibSongPath::new(DIR_PATH_STR))
                .times(1)
                .returning(|_, _| Ok(true));
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let result = t
                .get_path_by_path_str(&tx, &LibPathStr::from(DIR_PATH_STR.to_owned()))
                .unwrap();
            assert_eq!(result, vec![LibSongPath::new("test/hoge.flac")]);
        });
    }

    #[test]
    fn test_get_path_by_path_str_root() {
        fn expect() -> Vec<LibSongPath> {
            vec![
                LibSongPath::new("song1.mp3"),
                LibSongPath::new("test/hoge/song2.flac"),
                LibSongPath::new("test/song3.m4a"),
            ]
        }

        let mut mocks = Mocks::new();
        mocks.song_dao(|m| {
            m.expect_select_path_all()
                .times(1)
                .returning(|_| Ok(expect()));
            m.expect_exists_path()
                .withf(|_, a_path| a_path == &LibSongPath::new(""))
                .times(1)
                .returning(|_, _| Ok(false));
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let result = t.get_path_by_path_str(&tx, &LibPathStr::root()).unwrap();
            assert_eq!(result, expect());
        });
    }
}
