use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use super::DbTrackRepository;
use crate::{
    Error, FileLibraryRepository,
    artwork::DbArtworkRepository,
    db::DbTransaction,
    folder::{DbFolderRepository, FolderIdMayRoot, FolderUsecase},
    path::{LibPathStr, LibTrackPath, RelativeTrackPath},
    playlist::{DbPlaylistRepository, DbPlaylistTrackRepository},
    tag::DbTrackTagRepository,
};

/// 曲関係のUsecase
#[async_trait]
pub trait TrackUsecase {
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
    /// - track_path: 削除する曲のライブラリ内パス
    fn delete_track_pc(&self, pc_lib: &Path, track_path: &LibTrackPath) -> Result<()>;

    /// DAPから曲を削除
    ///
    /// # Arguments
    /// - dap_lib: DAPのライブラリルートパス
    /// - track_path: 削除する曲のライブラリ内パス
    fn delete_track_dap(&self, dap_lib: &Path, track_path: &LibTrackPath) -> Result<()>;

    /// DBから曲を削除
    ///
    /// # Arguments
    /// - path: 削除する曲のパス
    async fn delete_track_db<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibTrackPath,
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
    ) -> Result<Vec<LibTrackPath>>;
}

/// TrackUsecaseの本実装
#[allow(clippy::too_many_arguments)] // todo とりあえず後で整理
#[derive(new)]
pub struct TrackUsecaseImpl<FLR, AR, FR, PR, PSR, SR, STR, FU>
where
    FLR: FileLibraryRepository + Sync + Send,
    AR: DbArtworkRepository + Sync + Send,
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    PSR: DbPlaylistTrackRepository + Sync + Send,
    SR: DbTrackRepository + Sync + Send,
    STR: DbTrackTagRepository + Sync + Send,
    FU: FolderUsecase + Sync + Send,
{
    file_library_repository: FLR,
    db_artwork_repository: AR,
    db_folder_repository: FR,
    db_playlist_repository: PR,
    db_playlist_track_repository: PSR,
    db_track_repository: SR,
    db_track_tag_repository: STR,
    folder_usecase: FU,
}

#[async_trait]
impl<FLR, AR, FR, PR, PSR, SR, STR, FU> TrackUsecase
    for TrackUsecaseImpl<FLR, AR, FR, PR, PSR, SR, STR, FU>
where
    FLR: FileLibraryRepository + Sync + Send,
    AR: DbArtworkRepository + Sync + Send,
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    PSR: DbPlaylistTrackRepository + Sync + Send,
    SR: DbTrackRepository + Sync + Send,
    STR: DbTrackTagRepository + Sync + Send,
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
        let src_as_track = src.to_track_path();
        if self
            .db_track_repository
            .is_exist_path(tx, &src_as_track)
            .await?
        {
            self.move_track_db_unit(tx, &src_as_track, &dest.to_track_path())
                .await?;
        }
        //指定パス以下の全ての曲について、パスの変更を反映
        else {
            let src_as_dir = src.to_dir_path();
            let dest_as_dir = dest.to_dir_path();

            for src_track in self
                .db_track_repository
                .get_path_by_directory(tx, &src_as_dir)
                .await?
            {
                let relative_path =
                    RelativeTrackPath::from_track_and_parent(&src_track, &src_as_dir)?;
                let dest_track = relative_path.concat_lib_dir(&dest_as_dir);

                self.move_track_db_unit(tx, &src_track, &dest_track).await?;
            }
        };

        Ok(())
    }

    /// PCから曲を削除
    ///
    /// # Arguments
    /// - pc_lib: PCのライブラリルートパス
    /// - track_path: 削除する曲のライブラリ内パス
    fn delete_track_pc(&self, pc_lib: &Path, track_path: &LibTrackPath) -> Result<()> {
        //ゴミ箱へ
        self.file_library_repository.trash_track(pc_lib, track_path)
    }

    /// DAPから曲を削除
    ///
    /// # Arguments
    /// - dap_lib: DAPのライブラリルートパス
    /// - track_path: 削除する曲のライブラリ内パス
    fn delete_track_dap(&self, dap_lib: &Path, track_path: &LibTrackPath) -> Result<()> {
        self.file_library_repository
            .delete_track(dap_lib, track_path)
    }

    /// DBから曲を削除
    ///
    /// # Arguments
    /// - path: 削除する曲のパス
    async fn delete_track_db<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<()> {
        let db_track_repository = &self.db_track_repository;

        //ID情報を取得
        let track_id = db_track_repository
            .get_id_by_path(tx, path)
            .await?
            .ok_or_else(|| Error::DbTrackNotFound(path.clone()))?;

        //曲の削除
        db_track_repository.delete(tx, track_id).await?;

        //プレイリストからこの曲を削除
        self.db_playlist_track_repository
            .delete_track_from_all_playlists(tx, track_id)
            .await?;

        //タグと曲の紐付けを削除
        self.db_track_tag_repository
            .delete_all_tags_from_track(tx, track_id)
            .await?;

        //他に使用する曲がなければ、アートワークを削除
        self.db_artwork_repository
            .unregister_track_artworks(tx, track_id)
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
    ) -> Result<Vec<LibTrackPath>> {
        let track_path_list = self
            .db_track_repository
            .get_path_by_path_str(tx, path_str)
            .await?;

        for path in &track_path_list {
            self.delete_track_db(tx, path).await?;
        }

        Ok(track_path_list)
    }
}

impl<FLR, AR, FR, PR, PSR, SR, STR, FU> TrackUsecaseImpl<FLR, AR, FR, PR, PSR, SR, STR, FU>
where
    FLR: FileLibraryRepository + Sync + Send,
    AR: DbArtworkRepository + Sync + Send,
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    PSR: DbPlaylistTrackRepository + Sync + Send,
    SR: DbTrackRepository + Sync + Send,
    STR: DbTrackTagRepository + Sync + Send,
    FU: FolderUsecase + Sync + Send,
{
    /// 曲一つのDB内パス移動処理
    async fn move_track_db_unit<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        src: &LibTrackPath,
        dest: &LibTrackPath,
    ) -> Result<()> {
        if self.db_track_repository.is_exist_path(tx, dest).await? {
            return Err(Error::DbTrackAlreadyExists(dest.to_owned()).into());
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
        self.db_track_repository
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
pub struct MockTrackUsecase {
    pub inner: MockTrackUsecaseInner,
}
#[async_trait]
impl TrackUsecase for MockTrackUsecase {
    async fn move_path_str_db<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        src: &LibPathStr,
        dest: &LibPathStr,
    ) -> Result<()> {
        self.inner.move_path_str_db(src, dest)
    }

    fn delete_track_pc(&self, pc_lib: &Path, track_path: &LibTrackPath) -> Result<()> {
        self.inner.delete_track_pc(pc_lib, track_path)
    }

    fn delete_track_dap(&self, dap_lib: &Path, track_path: &LibTrackPath) -> Result<()> {
        self.inner.delete_track_dap(dap_lib, track_path)
    }

    async fn delete_track_db<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<()> {
        self.inner.delete_track_db(path)
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
    ) -> Result<Vec<LibTrackPath>> {
        self.inner.delete_path_str_db(path_str)
    }
}
mock! {
    pub TrackUsecaseInner {
        pub fn move_path_str_db(
            &self,
            src: &LibPathStr,
            dest: &LibPathStr,
        ) -> Result<()>;

        pub fn delete_track_pc(&self, pc_lib: &Path, track_path: &LibTrackPath) -> Result<()>;

        pub fn delete_track_dap(&self, dap_lib: &Path, track_path: &LibTrackPath) -> Result<()>;

        pub fn delete_track_db(&self, path: &LibTrackPath) -> Result<()>;

        pub fn delete_path_str_pc(&self, pc_lib: &Path, path_str: &LibPathStr) -> Result<()>;

        pub fn delete_path_str_dap(&self, dap_lib: &Path, path_str: &LibPathStr) -> Result<()>;

        pub fn delete_path_str_db(
            &self,
            path_str: &LibPathStr,
        ) -> Result<Vec<LibTrackPath>>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MockFileLibraryRepository,
        artwork::MockDbArtworkRepository,
        folder::{MockDbFolderRepository, MockFolderUsecase},
        path::LibDirPath,
        playlist::{MockDbPlaylistRepository, MockDbPlaylistTrackRepository},
        tag::MockDbTrackTagRepository,
        track::MockDbTrackRepository,
    };

    fn target() -> TrackUsecaseImpl<
        MockFileLibraryRepository,
        MockDbArtworkRepository,
        MockDbFolderRepository,
        MockDbPlaylistRepository,
        MockDbPlaylistTrackRepository,
        MockDbTrackRepository,
        MockDbTrackTagRepository,
        MockFolderUsecase,
    > {
        TrackUsecaseImpl {
            file_library_repository: MockFileLibraryRepository::default(),
            db_artwork_repository: MockDbArtworkRepository::default(),
            db_folder_repository: MockDbFolderRepository::default(),
            db_playlist_repository: MockDbPlaylistRepository::default(),
            db_playlist_track_repository: MockDbPlaylistTrackRepository::default(),
            db_track_repository: MockDbTrackRepository::default(),
            db_track_tag_repository: MockDbTrackTagRepository::default(),
            folder_usecase: MockFolderUsecase::default(),
        }
    }

    fn checkpoint_all(
        target: &mut TrackUsecaseImpl<
            MockFileLibraryRepository,
            MockDbArtworkRepository,
            MockDbFolderRepository,
            MockDbPlaylistRepository,
            MockDbPlaylistTrackRepository,
            MockDbTrackRepository,
            MockDbTrackTagRepository,
            MockFolderUsecase,
        >,
    ) {
        target.file_library_repository.checkpoint();
        target.db_artwork_repository.inner.checkpoint();
        target.db_folder_repository.inner.checkpoint();
        target.db_playlist_repository.inner.checkpoint();
        target.db_playlist_track_repository.inner.checkpoint();
        target.db_track_repository.inner.checkpoint();
        target.db_track_tag_repository.inner.checkpoint();
        target.folder_usecase.inner.checkpoint();
    }

    #[tokio::test]
    async fn test_delete_db_ok() -> anyhow::Result<()> {
        fn track_path() -> LibTrackPath {
            LibTrackPath::new("hoge/fuga.flac")
        }

        let mut target = target();
        target
            .db_track_repository
            .inner
            .expect_get_id_by_path()
            .withf(|a_path| a_path == &track_path())
            .returning(|_| Ok(Some(73)));
        target
            .db_track_repository
            .inner
            .expect_delete()
            .withf(|track_id| *track_id == 73)
            .times(1)
            .returning(|_| Ok(()));

        target
            .db_playlist_track_repository
            .inner
            .expect_delete_track_from_all_playlists()
            .withf(|track_id| *track_id == 73)
            .times(1)
            .returning(|_| Ok(()));

        target
            .db_track_tag_repository
            .inner
            .expect_delete_all_tags_from_track()
            .withf(|track_id| *track_id == 73)
            .times(1)
            .returning(|_| Ok(()));

        target
            .db_artwork_repository
            .inner
            .expect_unregister_track_artworks()
            .withf(|track_id| *track_id == 73)
            .times(1)
            .returning(|_| Ok(()));

        target
            .folder_usecase
            .inner
            .expect_delete_db_if_empty()
            .withf(|folder_path| folder_path == &LibDirPath::new("hoge"))
            .times(1)
            .returning(|_| Ok(()));

        target
            .db_playlist_repository
            .inner
            .expect_reset_listuped_flag()
            .times(1)
            .returning(|| Ok(()));

        let mut tx = DbTransaction::Dummy;

        target.delete_track_db(&mut tx, &track_path()).await?;

        checkpoint_all(&mut target);
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_db_no_track() -> anyhow::Result<()> {
        fn track_path() -> LibTrackPath {
            LibTrackPath::new("hoge.mp3")
        }

        let mut target = target();
        target
            .db_track_repository
            .inner
            .expect_get_id_by_path()
            .withf(|a_path| a_path == &track_path())
            .returning(|_| Ok(None));

        target.db_track_repository.inner.expect_delete().times(0);

        target
            .db_playlist_track_repository
            .inner
            .expect_delete_track_from_all_playlists()
            .times(0);

        target
            .db_track_tag_repository
            .inner
            .expect_delete_all_tags_from_track()
            .times(0);

        target
            .db_artwork_repository
            .inner
            .expect_unregister_track_artworks()
            .times(0);

        target
            .folder_usecase
            .inner
            .expect_delete_db_if_empty()
            .times(0);

        target
            .db_playlist_repository
            .inner
            .expect_reset_listuped_flag()
            .times(0);

        let mut tx = DbTransaction::Dummy;

        assert!(match target
            .delete_track_db(&mut tx, &track_path())
            .await
            .unwrap_err()
            .downcast_ref()
        {
            Some(Error::DbTrackNotFound(path)) => path == &track_path(),
            _ => false,
        });
        checkpoint_all(&mut target);
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_db_root_folder() -> anyhow::Result<()> {
        fn track_path() -> LibTrackPath {
            LibTrackPath::new("fuga.mp3")
        }

        let mut target = target();
        target
            .db_track_repository
            .inner
            .expect_get_id_by_path()
            .withf(|a_path| a_path == &track_path())
            .times(1)
            .returning(|_| Ok(Some(73)));
        target
            .db_track_repository
            .inner
            .expect_delete()
            .withf(|track_id| *track_id == 73)
            .times(1)
            .returning(|_| Ok(()));
        target
            .db_playlist_track_repository
            .inner
            .expect_delete_track_from_all_playlists()
            .withf(|track_id| *track_id == 73)
            .times(1)
            .returning(|_| Ok(()));

        target
            .db_track_tag_repository
            .inner
            .expect_delete_all_tags_from_track()
            .withf(|track_id| *track_id == 73)
            .times(1)
            .returning(|_| Ok(()));

        target
            .db_artwork_repository
            .inner
            .expect_unregister_track_artworks()
            .withf(|track_id| *track_id == 73)
            .times(1)
            .returning(|_| Ok(()));

        target
            .folder_usecase
            .inner
            .expect_delete_db_if_empty()
            .withf(|folder_path| folder_path == &LibDirPath::root())
            .times(1)
            .returning(|_| Ok(()));

        target
            .db_playlist_repository
            .inner
            .expect_reset_listuped_flag()
            .times(1)
            .returning(|| Ok(()));

        let mut tx = DbTransaction::Dummy;

        target.delete_track_db(&mut tx, &track_path()).await?;

        checkpoint_all(&mut target);

        Ok(())
    }
}
