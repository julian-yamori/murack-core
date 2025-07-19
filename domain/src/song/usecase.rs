use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use super::DbSongRepository;
use crate::{
    Error, FileLibraryRepository,
    artwork::DbArtworkRepository,
    db::DbTransaction,
    folder::{DbFolderRepository, FolderIdMayRoot, FolderUsecase},
    path::{LibPathStr, LibSongPath, RelativeSongPath},
    playlist::{DbPlaylistRepository, DbPlaylistSongRepository},
    tag::DbSongTagRepository,
};

/// 曲関係のUsecase
#[async_trait]
pub trait SongUsecase {
    /// パス文字列を指定してDBの曲パスを移動
    async fn move_path_str_db<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        src: &LibPathStr,
        dest: &LibPathStr,
    ) -> Result<()>;

    /// PCから曲を削除
    ///
    /// # Arguments
    /// - pc_lib: PCのライブラリルートパス
    /// - song_path: 削除する曲のライブラリ内パス
    fn delete_song_pc(&self, pc_lib: &Path, song_path: &LibSongPath) -> Result<()>;

    /// DAPから曲を削除
    ///
    /// # Arguments
    /// - dap_lib: DAPのライブラリルートパス
    /// - song_path: 削除する曲のライブラリ内パス
    fn delete_song_dap(&self, dap_lib: &Path, song_path: &LibSongPath) -> Result<()>;

    /// DBから曲を削除
    ///
    /// # Arguments
    /// - path: 削除する曲のパス
    async fn delete_song_db<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<()>;

    /// パス文字列を指定してPCから削除
    ///
    /// # Arguments
    /// - pc_lib: PCのライブラリルートパス
    /// - path_str: 削除するライブラリ内パス
    fn delete_path_str_pc(&self, pc_lib: &Path, path_str: &LibPathStr) -> Result<()>;

    /// パス文字列を指定してDAPから削除
    ///
    /// # Arguments
    /// - dap_lib: DAPのライブラリルートパス
    /// - path_str: 削除するライブラリ内パス
    fn delete_path_str_dap(&self, dap_lib: &Path, path_str: &LibPathStr) -> Result<()>;

    /// パス文字列を指定してDBから削除
    ///
    /// # Arguments
    /// - path: 削除する曲のパス
    ///
    /// # Returns
    /// 削除した曲のパスリスト
    async fn delete_path_str_db<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path_str: &LibPathStr,
    ) -> Result<Vec<LibSongPath>>;
}

/// SongUsecaseの本実装
#[allow(clippy::too_many_arguments)] // todo とりあえず後で整理
#[derive(new)]
pub struct SongUsecaseImpl<FLR, AR, FR, PR, PSR, SR, STR, FU>
where
    FLR: FileLibraryRepository + Sync + Send,
    AR: DbArtworkRepository + Sync + Send,
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    PSR: DbPlaylistSongRepository + Sync + Send,
    SR: DbSongRepository + Sync + Send,
    STR: DbSongTagRepository + Sync + Send,
    FU: FolderUsecase + Sync + Send,
{
    file_library_repository: FLR,
    db_artwork_repository: AR,
    db_folder_repository: FR,
    db_playlist_repository: PR,
    db_playlist_song_repository: PSR,
    db_song_repository: SR,
    db_song_tag_repository: STR,
    folder_usecase: FU,
}

#[async_trait]
impl<FLR, AR, FR, PR, PSR, SR, STR, FU> SongUsecase
    for SongUsecaseImpl<FLR, AR, FR, PR, PSR, SR, STR, FU>
where
    FLR: FileLibraryRepository + Sync + Send,
    AR: DbArtworkRepository + Sync + Send,
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    PSR: DbPlaylistSongRepository + Sync + Send,
    SR: DbSongRepository + Sync + Send,
    STR: DbSongTagRepository + Sync + Send,
    FU: FolderUsecase + Sync + Send,
{
    /// パス文字列を指定してDBの曲パスを移動
    async fn move_path_str_db<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        src: &LibPathStr,
        dest: &LibPathStr,
    ) -> Result<()> {
        //指定パスが曲ファイル自体なら、1曲だけ処理
        let src_as_song = src.to_song_path();
        if self
            .db_song_repository
            .is_exist_path(tx, &src_as_song)
            .await?
        {
            self.move_song_db_unit(tx, &src_as_song, &dest.to_song_path())
                .await?;
        }
        //指定パス以下の全ての曲について、パスの変更を反映
        else {
            let src_as_dir = src.to_dir_path();
            let dest_as_dir = dest.to_dir_path();

            for src_song in self
                .db_song_repository
                .get_path_by_directory(tx, &src_as_dir)
                .await?
            {
                let relative_path = RelativeSongPath::from_song_and_parent(&src_song, &src_as_dir)?;
                let dest_song = relative_path.concat_lib_dir(&dest_as_dir);

                self.move_song_db_unit(tx, &src_song, &dest_song).await?;
            }
        };

        Ok(())
    }

    /// PCから曲を削除
    ///
    /// # Arguments
    /// - pc_lib: PCのライブラリルートパス
    /// - song_path: 削除する曲のライブラリ内パス
    fn delete_song_pc(&self, pc_lib: &Path, song_path: &LibSongPath) -> Result<()> {
        //ゴミ箱へ
        self.file_library_repository.trash_song(pc_lib, song_path)
    }

    /// DAPから曲を削除
    ///
    /// # Arguments
    /// - dap_lib: DAPのライブラリルートパス
    /// - song_path: 削除する曲のライブラリ内パス
    fn delete_song_dap(&self, dap_lib: &Path, song_path: &LibSongPath) -> Result<()> {
        self.file_library_repository.delete_song(dap_lib, song_path)
    }

    /// DBから曲を削除
    ///
    /// # Arguments
    /// - path: 削除する曲のパス
    async fn delete_song_db<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<()> {
        let db_song_repository = &self.db_song_repository;

        //ID情報を取得
        let song_id = db_song_repository
            .get_id_by_path(tx, path)
            .await?
            .ok_or_else(|| Error::DbSongNotFound(path.clone()))?;

        //曲の削除
        db_song_repository.delete(tx, song_id).await?;

        //プレイリストからこの曲を削除
        self.db_playlist_song_repository
            .delete_song_from_all_playlists(tx, song_id)
            .await?;

        //タグと曲の紐付けを削除
        self.db_song_tag_repository
            .delete_all_tags_from_song(tx, song_id)
            .await?;

        //他に使用する曲がなければ、アートワークを削除
        self.db_artwork_repository
            .unregister_song_artworks(tx, song_id)
            .await?;

        //他に使用する曲がなければ、フォルダを削除
        self.folder_usecase
            .delete_db_if_empty(tx, &path.parent())
            .await?;

        self.db_playlist_repository.reset_listuped_flag(tx).await?;

        Ok(())
    }

    /// パス文字列を指定してPCから削除
    ///
    /// # Arguments
    /// - pc_lib: PCのライブラリルートパス
    /// - path_str: 削除するライブラリ内パス
    ///
    /// # Errors
    /// - alk_base_2_domain::Error::PathStrNotFoundPc: 指定されたパスが見つからなかった場合
    fn delete_path_str_pc(&self, pc_lib: &Path, path_str: &LibPathStr) -> Result<()> {
        self.file_library_repository
            .trash_path_str(pc_lib, path_str)
    }

    /// パス文字列を指定してDAPから削除
    ///
    /// # Arguments
    /// - dap_lib: DAPのライブラリルートパス
    /// - path_str: 削除するライブラリ内パス
    ///
    /// # Errors
    /// - alk_base_2_domain::Error::PathStrNotFoundDap: 指定されたパスが見つからなかった場合
    fn delete_path_str_dap(&self, dap_lib: &Path, path_str: &LibPathStr) -> Result<()> {
        self.file_library_repository
            .delete_path_str(dap_lib, path_str)
    }

    /// パス文字列を指定してDBから削除
    ///
    /// # Arguments
    /// - path: 削除する曲のパス
    ///
    /// # Returns
    /// 削除した曲のパスリスト
    async fn delete_path_str_db<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path_str: &LibPathStr,
    ) -> Result<Vec<LibSongPath>> {
        let song_path_list = self
            .db_song_repository
            .get_path_by_path_str(tx, path_str)
            .await?;

        for path in &song_path_list {
            self.delete_song_db(tx, path).await?;
        }

        Ok(song_path_list)
    }
}

impl<FLR, AR, FR, PR, PSR, SR, STR, FU> SongUsecaseImpl<FLR, AR, FR, PR, PSR, SR, STR, FU>
where
    FLR: FileLibraryRepository + Sync + Send,
    AR: DbArtworkRepository + Sync + Send,
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    PSR: DbPlaylistSongRepository + Sync + Send,
    SR: DbSongRepository + Sync + Send,
    STR: DbSongTagRepository + Sync + Send,
    FU: FolderUsecase + Sync + Send,
{
    /// 曲一つのDB内パス移動処理
    async fn move_song_db_unit<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        src: &LibSongPath,
        dest: &LibSongPath,
    ) -> Result<()> {
        if self.db_song_repository.is_exist_path(tx, dest).await? {
            return Err(Error::DbSongAlreadyExists(dest.to_owned()).into());
        }

        //移動先の親フォルダを登録してIDを取得
        let dest_parent = dest.parent();
        let new_folder_id = if dest_parent.is_root() {
            FolderIdMayRoot::Root
        } else {
            self.db_folder_repository
                .register_not_exists(tx, &dest_parent)
                .await?
        };

        //曲のパス情報を変更
        self.db_song_repository
            .update_path(tx, src, dest, new_folder_id)
            .await?;

        //子要素がなくなった親フォルダを削除
        self.folder_usecase
            .delete_db_if_empty(tx, &src.parent())
            .await?;

        //パスを使用したフィルタがあるかもしれないので、
        //プレイリストのリストアップ済みフラグを解除
        self.db_playlist_repository.reset_listuped_flag(tx).await?;
        //プレイリストファイル内のパスだけ変わるので、
        //DAP変更フラグを立てる
        self.db_playlist_repository
            .set_dap_change_flag_all(tx, true)
            .await?;

        Ok(())
    }
}

#[derive(Default)]
pub struct MockSongUsecase {
    pub inner: MockSongUsecaseInner,
}
#[async_trait]
impl SongUsecase for MockSongUsecase {
    async fn move_path_str_db<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        src: &LibPathStr,
        dest: &LibPathStr,
    ) -> Result<()> {
        self.inner.move_path_str_db(src, dest)
    }

    fn delete_song_pc(&self, pc_lib: &Path, song_path: &LibSongPath) -> Result<()> {
        self.inner.delete_song_pc(pc_lib, song_path)
    }

    fn delete_song_dap(&self, dap_lib: &Path, song_path: &LibSongPath) -> Result<()> {
        self.inner.delete_song_dap(dap_lib, song_path)
    }

    async fn delete_song_db<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<()> {
        self.inner.delete_song_db(path)
    }

    fn delete_path_str_pc(&self, pc_lib: &Path, path_str: &LibPathStr) -> Result<()> {
        self.inner.delete_path_str_pc(pc_lib, path_str)
    }

    fn delete_path_str_dap(&self, dap_lib: &Path, path_str: &LibPathStr) -> Result<()> {
        self.inner.delete_path_str_dap(dap_lib, path_str)
    }

    async fn delete_path_str_db<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path_str: &LibPathStr,
    ) -> Result<Vec<LibSongPath>> {
        self.inner.delete_path_str_db(path_str)
    }
}
mock! {
    pub SongUsecaseInner {
        pub fn move_path_str_db(
            &self,
            src: &LibPathStr,
            dest: &LibPathStr,
        ) -> Result<()>;

        pub fn delete_song_pc(&self, pc_lib: &Path, song_path: &LibSongPath) -> Result<()>;

        pub fn delete_song_dap(&self, dap_lib: &Path, song_path: &LibSongPath) -> Result<()>;

        pub fn delete_song_db(&self, path: &LibSongPath) -> Result<()>;

        pub fn delete_path_str_pc(&self, pc_lib: &Path, path_str: &LibPathStr) -> Result<()>;

        pub fn delete_path_str_dap(&self, dap_lib: &Path, path_str: &LibPathStr) -> Result<()>;

        pub fn delete_path_str_db(
            &self,
            path_str: &LibPathStr,
        ) -> Result<Vec<LibSongPath>>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MockFileLibraryRepository,
        artwork::MockDbArtworkRepository,
        folder::{MockDbFolderRepository, MockFolderUsecase},
        mocks,
        path::LibDirPath,
        playlist::{MockDbPlaylistRepository, MockDbPlaylistSongRepository},
        song::MockDbSongRepository,
        tag::MockDbSongTagRepository,
    };
    use paste::paste;

    mocks! {
        SongUsecaseImpl,
        [
            FileLibraryRepository,
            DbArtworkRepository,
            DbFolderRepository,
            DbPlaylistRepository,
            DbPlaylistSongRepository,
            DbSongRepository,
            DbSongTagRepository,
            FolderUsecase
        ]
    }

    #[test]
    fn test_delete_db_ok() {
        fn song_path() -> LibSongPath {
            LibSongPath::new("hoge/fuga.flac")
        }

        let mut mocks = Mocks::new();
        mocks.db_song_repository(|m| {
            m.expect_get_id_by_path()
                .withf(|_, a_path| a_path == &song_path())
                .returning(|_, _| Ok(Some(73)));
            m.expect_delete()
                .withf(|_, song_id| *song_id == 73)
                .times(1)
                .returning(|_, _| Ok(()));
        });
        mocks.db_playlist_song_repository(|m| {
            m.expect_delete_song_from_all_playlists()
                .withf(|_, song_id| *song_id == 73)
                .times(1)
                .returning(|_, _| Ok(()));
        });
        mocks.db_song_tag_repository(|m| {
            m.expect_delete_all_tags_from_song()
                .withf(|_, song_id| *song_id == 73)
                .times(1)
                .returning(|_, _| Ok(()));
        });
        mocks.db_artwork_repository(|m| {
            m.expect_unregister_song_artworks()
                .withf(|_, song_id| *song_id == 73)
                .times(1)
                .returning(|_, _| Ok(()));
        });
        mocks.folder_usecase(|m| {
            m.expect_delete_db_if_empty()
                .withf(|_, folder_path| folder_path == &LibDirPath::new("hoge"))
                .times(1)
                .returning(|_, _| Ok(()));
        });
        mocks.db_playlist_repository(|m| {
            m.expect_reset_listuped_flag()
                .times(1)
                .returning(|_| Ok(()));
        });

        let mut tx = DbTransaction::Dummy;

        mocks.run_target(|t| {
            t.delete_song_db(&mut tx, &song_path()).unwrap();
        });
    }
    #[test]
    fn test_delete_db_no_song() {
        fn song_path() -> LibSongPath {
            LibSongPath::new("hoge.mp3")
        }

        let mut mocks = Mocks::new();
        mocks.db_song_repository(|m| {
            m.expect_get_id_by_path()
                .withf(|_, a_path| a_path == &song_path())
                .returning(|_, _| Ok(None));
        });
        mocks.db_song_repository(|m| {
            m.expect_delete().times(0);
        });
        mocks.db_playlist_song_repository(|m| {
            m.expect_delete_song_from_all_playlists().times(0);
        });
        mocks.db_song_tag_repository(|m| {
            m.expect_delete_all_tags_from_song().times(0);
        });
        mocks.db_artwork_repository(|m| {
            m.expect_unregister_song_artworks().times(0);
        });
        mocks.folder_usecase(|m| {
            m.expect_delete_db_if_empty().times(0);
        });
        mocks.db_playlist_repository(|m| {
            m.expect_reset_listuped_flag().times(0);
        });

        let mut tx = DbTransaction::Dummy;

        mocks.run_target(|t| {
            assert!(match t
                .delete_song_db(&mut tx, &song_path())
                .unwrap_err()
                .downcast_ref()
            {
                Some(Error::DbSongNotFound(path)) => path == &song_path(),
                _ => false,
            });
        });
    }
    #[test]
    fn test_delete_db_root_folder() {
        fn song_path() -> LibSongPath {
            LibSongPath::new("fuga.mp3")
        }

        let mut mocks = Mocks::new();
        mocks.db_song_repository(|m| {
            m.expect_get_id_by_path()
                .withf(|_, a_path| a_path == &song_path())
                .times(1)
                .returning(|_, _| Ok(Some(73)));
            m.expect_delete()
                .withf(|_, song_id| *song_id == 73)
                .times(1)
                .returning(|_, _| Ok(()));
        });
        mocks.db_playlist_song_repository(|m| {
            m.expect_delete_song_from_all_playlists()
                .withf(|_, song_id| *song_id == 73)
                .times(1)
                .returning(|_, _| Ok(()));
        });
        mocks.db_song_tag_repository(|m| {
            m.expect_delete_all_tags_from_song()
                .withf(|_, song_id| *song_id == 73)
                .times(1)
                .returning(|_, _| Ok(()));
        });
        mocks.db_artwork_repository(|m| {
            m.expect_unregister_song_artworks()
                .withf(|_, song_id| *song_id == 73)
                .times(1)
                .returning(|_, _| Ok(()));
        });
        mocks.folder_usecase(|m| {
            m.expect_delete_db_if_empty()
                .withf(|_, folder_path| folder_path == &LibDirPath::root())
                .times(1)
                .returning(|_, _| Ok(()));
        });
        mocks.db_playlist_repository(|m| {
            m.expect_reset_listuped_flag()
                .times(1)
                .returning(|_| Ok(()));
        });

        let mut tx = DbTransaction::Dummy;

        mocks.run_target(|t| {
            t.delete_song_db(&mut tx, &song_path()).unwrap();
        });
    }
}
