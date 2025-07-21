use anyhow::Result;
use async_trait::async_trait;
use domain::{
    db::DbTransaction,
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibPathStr, LibSongPath},
    song::DbSongRepository,
};

use super::SongDao;

/// HasDbSongRepositoryの本実装
#[derive(new)]
pub struct DbSongRepositoryImpl<SD>
where
    SD: SongDao + Sync + Send,
{
    song_dao: SD,
}

#[async_trait]
impl<SD> DbSongRepository for DbSongRepositoryImpl<SD>
where
    SD: SongDao + Sync + Send,
{
    /// パスから曲IDを取得
    async fn get_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<i32>> {
        self.song_dao.select_id_by_path(tx, path).await
    }

    /// 文字列でパスを指定して、該当曲のパスリストを取得
    async fn get_path_by_path_str<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibPathStr,
    ) -> Result<Vec<LibSongPath>> {
        //ディレクトリ指定とみなして検索
        let dir_path = path.to_dir_path();
        let mut list = self.get_path_by_directory(tx, &dir_path).await?;

        //ファイル指定とみなしての検索でヒットしたら追加
        let song_path = path.to_song_path();
        if self.is_exist_path(tx, &song_path).await? {
            list.push(song_path);
        }

        Ok(list)
    }

    /// ディレクトリを指定してパスを取得
    /// # Arguments
    /// - path: 検索対象のライブラリパス
    /// # Returns
    /// 指定されたディレクトリ内の、全ての曲のパス
    async fn get_path_by_directory<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>> {
        //ルートフォルダ指定なら、全曲
        if path.is_root() {
            self.song_dao.select_path_all(tx).await
        } else {
            self.song_dao.select_path_begins_directory(tx, path).await
        }
    }

    /// ライブラリ内の全ての曲のパスを取得
    async fn get_path_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<Vec<LibSongPath>> {
        self.song_dao.select_path_all(tx).await
    }

    /// 指定したパスの曲が存在するか確認
    async fn is_exist_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<bool> {
        self.song_dao.exists_path(tx, path).await
    }

    /// 指定されたフォルダに曲が存在するか確認
    async fn is_exist_in_folder<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<bool> {
        let song_count = self
            .song_dao
            .count_by_folder_id(tx, FolderIdMayRoot::Folder(folder_id))
            .await?;
        Ok(song_count > 0)
    }

    /// 曲のパスを書き換え
    ///
    /// # Arguments
    /// - old_path: 書き換え元の曲のパス
    /// - new_path: 書き換え先の曲のパス
    /// - new_folder_id: 新しい親フォルダのID
    async fn update_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()> {
        self.song_dao
            .update_path_by_path(tx, old_path, new_path, new_folder_id)
            .await
    }

    /// 曲の再生時間を書き換え
    async fn update_duration<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()> {
        self.song_dao
            .update_duration_by_id(tx, song_id, duration)
            .await
    }

    /// 曲を削除
    ///
    /// # Arguments
    /// - song_id: 削除する曲のID
    async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, song_id: i32) -> Result<()> {
        self.song_dao.delete(tx, song_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::super::MockSongDao;
    use super::*;

    fn target() -> DbSongRepositoryImpl<MockSongDao> {
        DbSongRepositoryImpl {
            song_dao: MockSongDao::default(),
        }
    }
    fn checkpoint_all(target: &mut DbSongRepositoryImpl<MockSongDao>) {
        target.song_dao.inner.checkpoint();
    }

    #[tokio::test]
    async fn test_get_path_by_path_str_directory() -> anyhow::Result<()> {
        const DIR_PATH_STR: &str = "test/hoge";
        fn expect() -> Vec<LibSongPath> {
            vec![
                LibSongPath::new("test/hoge/song1.mp3"),
                LibSongPath::new("test/hoge/song2.flac"),
                LibSongPath::new("test/hoge/song3.m4a"),
            ]
        }

        let mut target = target();
        target
            .song_dao
            .inner
            .expect_select_path_begins_directory()
            .withf(|a_path| a_path == &LibDirPath::new(DIR_PATH_STR))
            .times(1)
            .returning(|_| Ok(expect()));
        target
            .song_dao
            .inner
            .expect_exists_path()
            .withf(|a_path| a_path == &LibSongPath::new(DIR_PATH_STR))
            .times(1)
            .returning(|_| Ok(false));

        let mut tx = DbTransaction::Dummy;

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::from(DIR_PATH_STR.to_owned()))
            .await?;
        assert_eq!(result, expect());

        checkpoint_all(&mut target);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_path_by_path_str_not_found() -> anyhow::Result<()> {
        const DIR_PATH_STR: &str = "test/hoge";

        let mut target = target();
        target
            .song_dao
            .inner
            .expect_select_path_begins_directory()
            .withf(|a_path| a_path == &LibDirPath::new(DIR_PATH_STR))
            .times(1)
            .returning(|_| Ok(vec![]));
        target
            .song_dao
            .inner
            .expect_exists_path()
            .withf(|a_path| a_path == &LibSongPath::new(DIR_PATH_STR))
            .times(1)
            .returning(|_| Ok(false));

        let mut tx = DbTransaction::Dummy;

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::from(DIR_PATH_STR.to_owned()))
            .await?;
        assert_eq!(result, vec![]);

        checkpoint_all(&mut target);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_path_by_path_str_song() -> anyhow::Result<()> {
        const DIR_PATH_STR: &str = "test/hoge.flac";

        let mut target = target();
        target
            .song_dao
            .inner
            .expect_select_path_begins_directory()
            .withf(|a_path| a_path == &LibDirPath::new(DIR_PATH_STR))
            .times(1)
            .returning(|_| Ok(vec![]));
        target
            .song_dao
            .inner
            .expect_exists_path()
            .withf(|a_path| a_path == &LibSongPath::new(DIR_PATH_STR))
            .times(1)
            .returning(|_| Ok(true));

        let mut tx = DbTransaction::Dummy;

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::from(DIR_PATH_STR.to_owned()))
            .await?;
        assert_eq!(result, vec![LibSongPath::new("test/hoge.flac")]);

        checkpoint_all(&mut target);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_path_by_path_str_root() -> anyhow::Result<()> {
        fn expect() -> Vec<LibSongPath> {
            vec![
                LibSongPath::new("song1.mp3"),
                LibSongPath::new("test/hoge/song2.flac"),
                LibSongPath::new("test/song3.m4a"),
            ]
        }

        let mut target = target();
        target
            .song_dao
            .inner
            .expect_select_path_all()
            .times(1)
            .returning(|| Ok(expect()));
        target
            .song_dao
            .inner
            .expect_exists_path()
            .withf(|a_path| a_path == &LibSongPath::new(""))
            .times(1)
            .returning(|_| Ok(false));

        let mut tx = DbTransaction::Dummy;

        let result = target
            .get_path_by_path_str(&mut tx, &LibPathStr::root())
            .await?;
        assert_eq!(result, expect());

        checkpoint_all(&mut target);
        Ok(())
    }
}
