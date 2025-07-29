use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use super::DbTrackRepository;
use crate::{
    Error, NonEmptyString,
    artwork::artwork_repository,
    folder::{FolderIdMayRoot, FolderUsecase, folder_repository},
    path::{LibraryDirectoryPath, LibraryTrackPath},
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
        path: &LibraryTrackPath,
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
    ) -> Result<Vec<LibraryTrackPath>>;
}

/// TrackUsecaseの本実装
#[derive(new)]
pub struct TrackUsecaseImpl<PR, PSR, SR, STR, FU>
where
    PR: DbPlaylistRepository + Sync + Send,
    PSR: DbPlaylistTrackRepository + Sync + Send,
    SR: DbTrackRepository + Sync + Send,
    STR: DbTrackTagRepository + Sync + Send,
    FU: FolderUsecase + Sync + Send,
{
    db_playlist_repository: PR,
    db_playlist_track_repository: PSR,
    db_track_repository: SR,
    db_track_tag_repository: STR,
    folder_usecase: FU,
}

#[async_trait]
impl<PR, PSR, SR, STR, FU> TrackUsecase for TrackUsecaseImpl<PR, PSR, SR, STR, FU>
where
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
        let src_as_track: LibraryTrackPath = src.clone().into();
        let track_exists = self
            .db_track_repository
            .is_exist_path(tx, &src_as_track)
            .await?;

        if track_exists {
            // 指定された 1 曲だけ処理

            let dest_as_track: LibraryTrackPath = dest.clone().into();

            self.move_track_db_unit(tx, &src_as_track, &dest_as_track)
                .await?;
        } else {
            // 指定ディレクトリ以下の全ての曲について、パスの変更を反映

            let src_as_dir: LibraryDirectoryPath = src.clone().into();
            let dest_as_dir: LibraryDirectoryPath = dest.clone().into();

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
        path: &LibraryTrackPath,
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
        artwork_repository::unregister_track_artworks(tx, track_id).await?;

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
    ) -> Result<Vec<LibraryTrackPath>> {
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

impl<PR, PSR, SR, STR, FU> TrackUsecaseImpl<PR, PSR, SR, STR, FU>
where
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
        src: &LibraryTrackPath,
        dest: &LibraryTrackPath,
    ) -> Result<()> {
        if self.db_track_repository.is_exist_path(tx, dest).await? {
            return Err(Error::DbTrackAlreadyExists(dest.to_owned()).into());
        }

        //移動先の親フォルダを登録してIDを取得
        let dest_parent_opt = dest.parent();
        let new_folder_id = match dest_parent_opt {
            None => FolderIdMayRoot::Root,
            Some(dest_parent) => {
                let id = folder_repository::register_not_exists(tx, &dest_parent).await?;
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
    src_track: &LibraryTrackPath,
    src_dir: &LibraryDirectoryPath,
    dest_dir: &LibraryDirectoryPath,
) -> anyhow::Result<LibraryTrackPath> {
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
        path: &LibraryTrackPath,
    ) -> Result<()> {
        self.inner.delete_track_db(path)
    }

    async fn delete_path_str_db<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        path_str: &NonEmptyString,
    ) -> Result<Vec<LibraryTrackPath>> {
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

        pub fn delete_track_pc(&self, pc_lib: &Path, track_path: &LibraryTrackPath) -> Result<()>;

        pub fn delete_track_dap(&self, dap_lib: &Path, track_path: &LibraryTrackPath) -> Result<()>;

        pub fn delete_track_db(&self, path: &LibraryTrackPath) -> Result<()>;

        pub fn delete_path_str_pc(&self, pc_lib: &Path, path_str: &NonEmptyString) -> Result<()>;

        pub fn delete_path_str_dap(&self, dap_lib: &Path, path_str: &NonEmptyString) -> Result<()>;

        pub fn delete_path_str_db(
            &self,
            path_str: &NonEmptyString,
        ) -> Result<Vec<LibraryTrackPath>>;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use test_case::test_case;

    use super::*;
    use crate::path::LibraryDirectoryPath;

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
            &LibraryTrackPath::from_str(src_track)?,
            &LibraryDirectoryPath::from_str(src_dir)?,
            &LibraryDirectoryPath::from_str(dest_dir)?,
        )?;

        assert_eq!(actual.as_ref() as &str, expect_dest_track);
        Ok(())
    }
}
