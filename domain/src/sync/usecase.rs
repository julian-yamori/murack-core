use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use super::{DbSongSyncRepository, SongSync};
use crate::{
    db::DbTransaction,
    folder::{DbFolderRepository, FolderIdMayRoot},
    path::LibSongPath,
    playlist::DbPlaylistRepository,
};

/// DB・PC連携のUseCase
#[async_trait]
pub trait SyncUsecase {
    /// DBに曲データを新規登録する
    ///
    /// # Arguments
    /// - db: DB接続
    /// - song_path: 登録する曲のライブラリ内パス
    /// - song_sync: 登録する曲のデータ
    /// - entry_date: 登録日
    async fn register_db<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_path: &LibSongPath,
        song_sync: &mut SongSync,
    ) -> Result<()>;
}

/// SyncUsecaseの本実装
#[derive(new)]
pub struct SyncUsecaseImpl<FR, PR, SSR>
where
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    SSR: DbSongSyncRepository + Sync + Send,
{
    db_folder_repository: FR,
    db_playlist_repository: PR,
    db_song_sync_repository: SSR,
}
#[async_trait]
impl<FR, PR, SSR> SyncUsecase for SyncUsecaseImpl<FR, PR, SSR>
where
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    SSR: DbSongSyncRepository + Sync + Send,
{
    /// DBに曲データを新規登録する
    ///
    /// # Arguments
    /// - db: DB接続
    /// - song_path: 登録する曲のライブラリ内パス
    /// - song_sync: 登録する曲のデータ
    /// - entry_date: 登録日
    async fn register_db<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_path: &LibSongPath,
        song_sync: &mut SongSync,
    ) -> Result<()> {
        //曲名が空なら、ファイル名から取得
        if song_sync.title.is_none() {
            song_sync.title = Some(song_path.file_stem().to_owned());
        };

        //親ディレクトリを登録してIDを取得
        let parent_path = song_path.parent();
        let folder_id = if parent_path.is_root() {
            FolderIdMayRoot::Root
        } else {
            self.db_folder_repository
                .register_not_exists(tx, &parent_path)
                .await?
        };

        //DBに書き込み
        self.db_song_sync_repository
            .register(tx, song_path, song_sync, folder_id)
            .await?;

        //プレイリストのリストアップ済みフラグを解除
        self.db_playlist_repository.reset_listuped_flag(tx).await?;

        Ok(())
    }
}

#[derive(Default)]
pub struct MockSyncUsecase {
    pub inner: MockSyncUsecaseInner,
}
#[async_trait]
impl SyncUsecase for MockSyncUsecase {
    async fn register_db<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        song_path: &LibSongPath,
        song_sync: &mut SongSync,
    ) -> Result<()> {
        self.inner.register_db(song_path, song_sync)
    }
}
mock! {
    pub SyncUsecaseInner {
        pub fn register_db(
            &self,
            song_path: &LibSongPath,
            song_sync: &mut SongSync,
        ) -> Result<()>;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::NaiveDate;
    use murack_core_media::picture::Picture;

    use super::super::MockDbSongSyncRepository;
    use super::*;
    use crate::{
        artwork::SongArtwork, folder::MockDbFolderRepository, path::LibDirPath,
        playlist::MockDbPlaylistRepository,
    };

    fn target()
    -> SyncUsecaseImpl<MockDbFolderRepository, MockDbPlaylistRepository, MockDbSongSyncRepository>
    {
        SyncUsecaseImpl {
            db_folder_repository: MockDbFolderRepository::default(),
            db_playlist_repository: MockDbPlaylistRepository::default(),
            db_song_sync_repository: MockDbSongSyncRepository::default(),
        }
    }
    fn checkpoint_all(
        target: &mut SyncUsecaseImpl<
            MockDbFolderRepository,
            MockDbPlaylistRepository,
            MockDbSongSyncRepository,
        >,
    ) {
        target.db_folder_repository.inner.checkpoint();
        target.db_playlist_repository.inner.checkpoint();
        target.db_song_sync_repository.inner.checkpoint();
    }

    fn song_sync() -> SongSync {
        SongSync {
            duration: 120000,
            title: Some("曲名".to_owned()),
            artist: Some("アーティスト".to_owned()),
            album: Some("アルバむ".to_owned()),
            genre: Some("Genre".to_owned()),
            album_artist: Some("".to_owned()),
            composer: Some("".to_owned()),
            track_number: Some(1),
            track_max: Some(2),
            disc_number: Some(3),
            disc_max: Some(4),
            release_date: Some(NaiveDate::from_ymd_opt(2013, 7, 14).unwrap()),
            memo: Some("メモ".to_owned()),
            lyrics: Some("歌詞".to_owned()),
            artworks: vec![SongArtwork {
                picture: Arc::new(Picture {
                    bytes: vec![1, 2, 3, 4],
                    mime_type: "image/jpeg".to_owned(),
                }),
                picture_type: 3,
                description: "説明".to_owned(),
            }],
        }
    }

    #[tokio::test]
    async fn test_register_db_root_folder() -> anyhow::Result<()> {
        fn song_path() -> LibSongPath {
            LibSongPath::new("song.flac")
        }

        let mut target = target();
        target
            .db_folder_repository
            .inner
            .expect_register_not_exists()
            .times(0);

        target
            .db_song_sync_repository
            .inner
            .expect_register()
            .times(1)
            .returning(|_, a_song_sync, a_folder_id| {
                assert_eq!(a_folder_id, FolderIdMayRoot::Root);
                assert_eq!(a_song_sync.title.as_deref(), Some("曲名"));
                Ok(5)
            });

        target
            .db_playlist_repository
            .inner
            .expect_reset_listuped_flag()
            .times(1)
            .returning(|| Ok(()));

        let mut tx = DbTransaction::Dummy;

        let mut s = song_sync();
        target.register_db(&mut tx, &song_path(), &mut s).await?;

        assert_eq!(s.title.as_deref(), Some("曲名"));

        checkpoint_all(&mut target);
        Ok(())
    }

    #[tokio::test]
    async fn test_register_db_no_title() -> anyhow::Result<()> {
        fn song_path() -> LibSongPath {
            LibSongPath::new("test/hoge/fuga.mp3")
        }

        let mut target = target();
        target
            .db_folder_repository
            .inner
            .expect_register_not_exists()
            .times(1)
            .returning(|a_path| {
                assert_eq!(a_path, &LibDirPath::new("test/hoge"));
                Ok(FolderIdMayRoot::Folder(15))
            });

        target
            .db_song_sync_repository
            .inner
            .expect_register()
            .times(1)
            .returning(|_, a_song_sync, a_folder_id| {
                assert_eq!(a_folder_id, FolderIdMayRoot::Folder(15));
                assert_eq!(a_song_sync.title.as_deref(), Some("fuga"));
                Ok(5)
            });

        target
            .db_playlist_repository
            .inner
            .expect_reset_listuped_flag()
            .times(1)
            .returning(|| Ok(()));

        let mut tx = DbTransaction::Dummy;

        let mut s = song_sync();
        s.title = None;

        target.register_db(&mut tx, &song_path(), &mut s).await?;

        assert_eq!(s.title.as_deref(), Some("fuga"));

        checkpoint_all(&mut target);
        Ok(())
    }
}
