use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use super::{DbTrackSyncRepository, TrackSync};
use crate::{
    folder::{DbFolderRepository, FolderIdMayRoot},
    path::LibTrackPath,
    playlist::DbPlaylistRepository,
};
use sqlx::PgTransaction;

/// DB・PC連携のUseCase
#[async_trait]
pub trait SyncUsecase {
    /// DBに曲データを新規登録する
    ///
    /// # Arguments
    /// - db: DB接続
    /// - track_path: 登録する曲のライブラリ内パス
    /// - track_sync: 登録する曲のデータ
    /// - entry_date: 登録日
    async fn register_db<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track_path: &LibTrackPath,
        track_sync: &mut TrackSync,
    ) -> Result<()>;
}

/// SyncUsecaseの本実装
#[derive(new)]
pub struct SyncUsecaseImpl<FR, PR, SSR>
where
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    SSR: DbTrackSyncRepository + Sync + Send,
{
    db_folder_repository: FR,
    db_playlist_repository: PR,
    db_track_sync_repository: SSR,
}
#[async_trait]
impl<FR, PR, SSR> SyncUsecase for SyncUsecaseImpl<FR, PR, SSR>
where
    FR: DbFolderRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    SSR: DbTrackSyncRepository + Sync + Send,
{
    /// DBに曲データを新規登録する
    ///
    /// # Arguments
    /// - db: DB接続
    /// - track_path: 登録する曲のライブラリ内パス
    /// - track_sync: 登録する曲のデータ
    /// - entry_date: 登録日
    async fn register_db<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track_path: &LibTrackPath,
        track_sync: &mut TrackSync,
    ) -> Result<()> {
        //曲名が空なら、ファイル名から取得
        if track_sync.title.is_empty() {
            track_sync.title = track_path.file_stem().to_owned();
        };

        //親ディレクトリを登録してIDを取得
        let parent_path_opt = track_path.parent();
        let folder_id = match parent_path_opt {
            None => FolderIdMayRoot::Root,
            Some(parent_path) => {
                self.db_folder_repository
                    .register_not_exists(tx, &parent_path)
                    .await?
            }
        };

        //DBに書き込み
        self.db_track_sync_repository
            .register(tx, track_path, track_sync, folder_id)
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
        _db: &mut PgTransaction<'c>,
        track_path: &LibTrackPath,
        track_sync: &mut TrackSync,
    ) -> Result<()> {
        self.inner.register_db(track_path, track_sync)
    }
}
mock! {
    pub SyncUsecaseInner {
        pub fn register_db(
            &self,
            track_path: &LibTrackPath,
            track_sync: &mut TrackSync,
        ) -> Result<()>;
    }
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::Arc};

    use chrono::NaiveDate;
    use murack_core_media::picture::Picture;
    use sqlx::PgPool;

    use super::super::MockDbTrackSyncRepository;
    use super::*;
    use crate::{
        artwork::TrackArtwork, folder::MockDbFolderRepository, path::LibDirPath,
        playlist::MockDbPlaylistRepository,
    };

    fn target()
    -> SyncUsecaseImpl<MockDbFolderRepository, MockDbPlaylistRepository, MockDbTrackSyncRepository>
    {
        SyncUsecaseImpl {
            db_folder_repository: MockDbFolderRepository::default(),
            db_playlist_repository: MockDbPlaylistRepository::default(),
            db_track_sync_repository: MockDbTrackSyncRepository::default(),
        }
    }
    fn checkpoint_all(
        target: &mut SyncUsecaseImpl<
            MockDbFolderRepository,
            MockDbPlaylistRepository,
            MockDbTrackSyncRepository,
        >,
    ) {
        target.db_folder_repository.inner.checkpoint();
        target.db_playlist_repository.inner.checkpoint();
        target.db_track_sync_repository.inner.checkpoint();
    }

    fn track_sync() -> TrackSync {
        TrackSync {
            duration: 120000,
            title: "曲名".to_owned(),
            artist: "アーティスト".to_owned(),
            album: "アルバむ".to_owned(),
            genre: "Genre".to_owned(),
            album_artist: "".to_owned(),
            composer: "".to_owned(),
            track_number: Some(1),
            track_max: Some(2),
            disc_number: Some(3),
            disc_max: Some(4),
            release_date: Some(NaiveDate::from_ymd_opt(2013, 7, 14).unwrap()),
            memo: "メモ".to_owned(),
            lyrics: "歌詞".to_owned(),
            artworks: vec![TrackArtwork {
                picture: Arc::new(Picture {
                    bytes: vec![1, 2, 3, 4],
                    mime_type: "image/jpeg".to_owned(),
                }),
                picture_type: 3,
                description: "説明".to_owned(),
            }],
        }
    }

    #[sqlx::test]
    async fn test_register_db_root_folder(pool: PgPool) -> anyhow::Result<()> {
        fn track_path() -> LibTrackPath {
            LibTrackPath::from_str("track.flac").unwrap()
        }

        let mut target = target();
        target
            .db_folder_repository
            .inner
            .expect_register_not_exists()
            .times(0);

        target
            .db_track_sync_repository
            .inner
            .expect_register()
            .times(1)
            .returning(|_, a_track_sync, a_folder_id| {
                assert_eq!(a_folder_id, FolderIdMayRoot::Root);
                assert_eq!(&a_track_sync.title, "曲名");
                Ok(5)
            });

        target
            .db_playlist_repository
            .inner
            .expect_reset_listuped_flag()
            .times(1)
            .returning(|| Ok(()));

        let mut tx = pool.begin().await?;

        let mut s = track_sync();
        target.register_db(&mut tx, &track_path(), &mut s).await?;

        assert_eq!(&s.title, "曲名");

        checkpoint_all(&mut target);
        Ok(())
    }

    #[sqlx::test]
    async fn test_register_db_no_title(pool: PgPool) -> anyhow::Result<()> {
        fn track_path() -> LibTrackPath {
            LibTrackPath::from_str("test/hoge/fuga.mp3").unwrap()
        }

        let mut target = target();
        target
            .db_folder_repository
            .inner
            .expect_register_not_exists()
            .times(1)
            .returning(|a_path| {
                assert_eq!(a_path, &LibDirPath::from_str("test/hoge").unwrap());
                Ok(FolderIdMayRoot::Folder(15))
            });

        target
            .db_track_sync_repository
            .inner
            .expect_register()
            .times(1)
            .returning(|_, a_track_sync, a_folder_id| {
                assert_eq!(a_folder_id, FolderIdMayRoot::Folder(15));
                assert_eq!(&a_track_sync.title, "fuga");
                Ok(5)
            });

        target
            .db_playlist_repository
            .inner
            .expect_reset_listuped_flag()
            .times(1)
            .returning(|| Ok(()));

        let mut tx = pool.begin().await?;

        let mut s = track_sync();
        s.title = String::default();

        target.register_db(&mut tx, &track_path(), &mut s).await?;

        assert_eq!(&s.title, "fuga");

        checkpoint_all(&mut target);
        Ok(())
    }
}
