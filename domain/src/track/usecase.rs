use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use super::DbTrackRepository;
use crate::{
    Error, NonEmptyString,
    artwork::DbArtworkRepository,
    folder::{DbFolderRepository, FolderIdMayRoot, FolderUsecase},
    path::{LibDirPath, LibTrackPath},
    playlist::{DbPlaylistRepository, DbPlaylistTrackRepository},
    tag::DbTrackTagRepository,
};
use sqlx::PgTransaction;

/// 曲関係のUsecase
#[async_trait]
pub trait TrackUsecase {
    /// パス文字列を指定してDBの曲パスを移動
    async fn move_path_str_db<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        src: &NonEmptyString,
        dest: &NonEmptyString,
    ) -> Result<()>;

    /// DBから曲を削除
    ///
    /// # Arguments
    /// - path: 削除する曲のパス
    async fn delete_track_db<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<()>;

    /// パス文字列を指定してDBから削除
    ///
    /// # Arguments
    /// - path: 削除する曲のパス
    ///
    /// # Returns
    /// 削除した曲のパスリスト
    async fn delete_path_str_db<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path_str: &NonEmptyString,
    ) -> Result<Vec<LibTrackPath>>;
}

/// TrackUsecaseの本実装
#[derive(new)]
pub struct TrackUsecaseImpl<AR, FR, PR, PSR, SR, STR, FU>
where
    AR: DbArtworkRepository + Sync + Send,
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    PSR: DbPlaylistTrackRepository + Sync + Send,
    SR: DbTrackRepository + Sync + Send,
    STR: DbTrackTagRepository + Sync + Send,
    FU: FolderUsecase + Sync + Send,
{
    db_artwork_repository: AR,
    db_folder_repository: FR,
    db_playlist_repository: PR,
    db_playlist_track_repository: PSR,
    db_track_repository: SR,
    db_track_tag_repository: STR,
    folder_usecase: FU,
}

#[async_trait]
impl<AR, FR, PR, PSR, SR, STR, FU> TrackUsecase for TrackUsecaseImpl<AR, FR, PR, PSR, SR, STR, FU>
where
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
        tx: &mut PgTransaction<'c>,
        src: &NonEmptyString,
        dest: &NonEmptyString,
    ) -> Result<()> {
        // パス文字列がファイルかどうかを、完全一致するパスの曲が DB に存在するかどうかで判定
        let src_as_track: LibTrackPath = src.clone().into();
        let track_exists = self
            .db_track_repository
            .is_exist_path(tx, &src_as_track)
            .await?;

        if track_exists {
            // 指定された 1 曲だけ処理

            let dest_as_track: LibTrackPath = dest.clone().into();

            self.move_track_db_unit(tx, &src_as_track, &dest_as_track)
                .await?;
        } else {
            // 指定ディレクトリ以下の全ての曲について、パスの変更を反映

            let src_as_dir: LibDirPath = src.clone().into();
            let dest_as_dir: LibDirPath = dest.clone().into();

            for src_track in self
                .db_track_repository
                .get_path_by_directory(tx, &src_as_dir)
                .await?
            {
                let dest_track = src_child_path_to_dest(&src_track, &src_as_dir, &dest_as_dir)?;
                self.move_track_db_unit(tx, &src_track, &dest_track).await?;
            }
        };

        Ok(())
    }

    /// DBから曲を削除
    ///
    /// # Arguments
    /// - path: 削除する曲のパス
    async fn delete_track_db<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
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

        //他に使用する曲がなければ、親フォルダを削除
        if let Some(parent) = path.parent() {
            self.folder_usecase.delete_db_if_empty(tx, &parent).await?;
        };

        self.db_playlist_repository.reset_listuped_flag(tx).await?;

        Ok(())
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
        tx: &mut PgTransaction<'c>,
        path_str: &NonEmptyString,
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

impl<AR, FR, PR, PSR, SR, STR, FU> TrackUsecaseImpl<AR, FR, PR, PSR, SR, STR, FU>
where
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
        tx: &mut PgTransaction<'c>,
        src: &LibTrackPath,
        dest: &LibTrackPath,
    ) -> Result<()> {
        if self.db_track_repository.is_exist_path(tx, dest).await? {
            return Err(Error::DbTrackAlreadyExists(dest.to_owned()).into());
        }

        //移動先の親フォルダを登録してIDを取得
        let dest_parent_opt = dest.parent();
        let new_folder_id = match dest_parent_opt {
            None => FolderIdMayRoot::Root,
            Some(dest_parent) => {
                let id = self
                    .db_folder_repository
                    .register_not_exists(tx, &dest_parent)
                    .await?;
                FolderIdMayRoot::Folder(id)
            }
        };

        //曲のパス情報を変更
        self.db_track_repository
            .update_path(tx, src, dest, new_folder_id)
            .await?;

        //子要素がなくなった親フォルダを削除
        if let Some(parent) = src.parent() {
            self.folder_usecase.delete_db_if_empty(tx, &parent).await?;
        }

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

/// move コマンドで指定された src_dir の子の src_track から、dest_dir の子として移動する先のパスを取得
fn src_child_path_to_dest(
    src_track: &LibTrackPath,
    src_dir: &LibDirPath,
    dest_dir: &LibDirPath,
) -> anyhow::Result<LibTrackPath> {
    let src_dir_str: &str = src_dir.as_ref();
    let src_track_str: &str = src_track.as_ref();

    // src_track が src_dir で始まっているか確認
    if !src_track_str.starts_with(src_dir_str) {
        return Err(Error::GetRelativePathFailed {
            track: src_track.to_owned(),
            parent: src_dir.to_owned(),
        }
        .into());
    }

    let relative_path = &src_track_str[src_dir_str.len()..];

    // 文字列を取得して連結
    let mut s = (dest_dir.as_ref() as &NonEmptyString).clone();
    s.push_str(relative_path);

    Ok(s.into())
}

#[derive(Default)]
pub struct MockTrackUsecase {
    pub inner: MockTrackUsecaseInner,
}
#[async_trait]
impl TrackUsecase for MockTrackUsecase {
    async fn move_path_str_db<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        src: &NonEmptyString,
        dest: &NonEmptyString,
    ) -> Result<()> {
        self.inner.move_path_str_db(src, dest)
    }

    async fn delete_track_db<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<()> {
        self.inner.delete_track_db(path)
    }

    async fn delete_path_str_db<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        path_str: &NonEmptyString,
    ) -> Result<Vec<LibTrackPath>> {
        self.inner.delete_path_str_db(path_str)
    }
}
mock! {
    pub TrackUsecaseInner {
        pub fn move_path_str_db(
            &self,
            src: &NonEmptyString,
            dest: &NonEmptyString,
        ) -> Result<()>;

        pub fn delete_track_pc(&self, pc_lib: &Path, track_path: &LibTrackPath) -> Result<()>;

        pub fn delete_track_dap(&self, dap_lib: &Path, track_path: &LibTrackPath) -> Result<()>;

        pub fn delete_track_db(&self, path: &LibTrackPath) -> Result<()>;

        pub fn delete_path_str_pc(&self, pc_lib: &Path, path_str: &NonEmptyString) -> Result<()>;

        pub fn delete_path_str_dap(&self, dap_lib: &Path, path_str: &NonEmptyString) -> Result<()>;

        pub fn delete_path_str_db(
            &self,
            path_str: &NonEmptyString,
        ) -> Result<Vec<LibTrackPath>>;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sqlx::PgPool;
    use test_case::test_case;

    use super::*;
    use crate::{
        artwork::MockDbArtworkRepository,
        folder::{MockDbFolderRepository, MockFolderUsecase},
        path::LibDirPath,
        playlist::{MockDbPlaylistRepository, MockDbPlaylistTrackRepository},
        tag::MockDbTrackTagRepository,
        track::MockDbTrackRepository,
    };

    fn target() -> TrackUsecaseImpl<
        MockDbArtworkRepository,
        MockDbFolderRepository,
        MockDbPlaylistRepository,
        MockDbPlaylistTrackRepository,
        MockDbTrackRepository,
        MockDbTrackTagRepository,
        MockFolderUsecase,
    > {
        TrackUsecaseImpl {
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
            MockDbArtworkRepository,
            MockDbFolderRepository,
            MockDbPlaylistRepository,
            MockDbPlaylistTrackRepository,
            MockDbTrackRepository,
            MockDbTrackTagRepository,
            MockFolderUsecase,
        >,
    ) {
        target.db_artwork_repository.inner.checkpoint();
        target.db_folder_repository.inner.checkpoint();
        target.db_playlist_repository.inner.checkpoint();
        target.db_playlist_track_repository.inner.checkpoint();
        target.db_track_repository.inner.checkpoint();
        target.db_track_tag_repository.inner.checkpoint();
        target.folder_usecase.inner.checkpoint();
    }

    #[sqlx::test]
    async fn test_delete_db_ok(pool: PgPool) -> anyhow::Result<()> {
        fn track_path() -> LibTrackPath {
            LibTrackPath::try_from("hoge/fuga.flac".to_string()).unwrap()
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
            .withf(|folder_path| folder_path == &LibDirPath::from_str("hoge").unwrap())
            .times(1)
            .returning(|_| Ok(()));

        target
            .db_playlist_repository
            .inner
            .expect_reset_listuped_flag()
            .times(1)
            .returning(|| Ok(()));

        let mut tx = pool.begin().await?;

        target.delete_track_db(&mut tx, &track_path()).await?;

        checkpoint_all(&mut target);
        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_db_no_track(pool: PgPool) -> anyhow::Result<()> {
        fn track_path() -> LibTrackPath {
            LibTrackPath::from_str("hoge.mp3").unwrap()
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

        let mut tx = pool.begin().await?;

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

    #[sqlx::test]
    async fn test_delete_db_root_folder(pool: PgPool) -> anyhow::Result<()> {
        fn track_path() -> LibTrackPath {
            LibTrackPath::from_str("fuga.mp3").unwrap()
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
            .never();

        target
            .db_playlist_repository
            .inner
            .expect_reset_listuped_flag()
            .times(1)
            .returning(|| Ok(()));

        let mut tx = pool.begin().await?;

        target.delete_track_db(&mut tx, &track_path()).await?;

        checkpoint_all(&mut target);

        Ok(())
    }

    #[test_case("parent/src/dir/file.flac", "parent/src", "parent", "parent/dir/file.flac"; "1階層上に移動")]
    #[test_case("parent/dir/file.flac", "parent", "parent/dest", "parent/dest/dir/file.flac"; "親ディレクトリの1階層下に移動")]
    #[test_case("src/dir/file.flac", "src", "parent/dest", "parent/dest/dir/file.flac"; "別ディレクトリの1階層下に移動")]
    #[test_case("parent/src/file.flac", "parent/src", "parent/dest", "parent/dest/file.flac"; "子ディレクトリなし")]
    #[test_case("src/dir/dir2/file.flac", "src", "dest", "dest/dir/dir2/file.flac"; "子ディレクトリが2階層")]
    fn test_src_child_path_to_dest(
        src_track: &str,
        src_dir: &str,
        dest_dir: &str,
        expect_dest_track: &str,
    ) -> anyhow::Result<()> {
        let actual = src_child_path_to_dest(
            &LibTrackPath::from_str(src_track)?,
            &LibDirPath::from_str(src_dir)?,
            &LibDirPath::from_str(dest_dir)?,
        )?;

        assert_eq!(actual.as_ref() as &str, expect_dest_track);
        Ok(())
    }
}
