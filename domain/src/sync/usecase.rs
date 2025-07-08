use super::{DbSongSyncRepository, SongSync};
use crate::{
    db_wrapper::TransactionWrapper,
    folder::{DbFolderRepository, FolderIdMayRoot},
    path::LibSongPath,
    playlist::DbPlaylistRepository,
};
use anyhow::Result;
use chrono::NaiveDate;
use mockall::automock;
use std::rc::Rc;

/// DB・PC連携のUseCase
#[automock]
pub trait SyncUsecase {
    /// DBに曲データを新規登録する
    ///
    /// # Arguments
    /// - db: DB接続
    /// - song_path: 登録する曲のライブラリ内パス
    /// - song_sync: 登録する曲のデータ
    /// - entry_date: 登録日
    fn register_db<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_path: &LibSongPath,
        song_sync: &mut SongSync,
        entry_date: NaiveDate,
    ) -> Result<()>;
}

/// SyncUsecaseの本実装
#[derive(new)]
pub struct SyncUsecaseImpl {
    db_folder_repository: Rc<dyn DbFolderRepository>,
    db_playlist_repository: Rc<dyn DbPlaylistRepository>,
    db_song_sync_repository: Rc<dyn DbSongSyncRepository>,
}
impl SyncUsecase for SyncUsecaseImpl {
    /// DBに曲データを新規登録する
    ///
    /// # Arguments
    /// - db: DB接続
    /// - song_path: 登録する曲のライブラリ内パス
    /// - song_sync: 登録する曲のデータ
    /// - entry_date: 登録日
    fn register_db(
        &self,
        tx: &TransactionWrapper,
        song_path: &LibSongPath,
        song_sync: &mut SongSync,
        entry_date: NaiveDate,
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
                .register_not_exists(tx, &parent_path)?
        };

        //DBに書き込み
        self.db_song_sync_repository
            .register(tx, song_path, song_sync, folder_id, entry_date)?;

        //プレイリストのリストアップ済みフラグを解除
        self.db_playlist_repository.reset_listuped_flag(tx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::MockDbSongSyncRepository;
    use super::*;
    use crate::{
        artwork::SongArtwork, db_wrapper::ConnectionFactory, folder::MockDbFolderRepository, mocks,
        path::LibDirPath, playlist::MockDbPlaylistRepository,
    };
    use media::picture::Picture;
    use paste::paste;

    mocks! {
        SyncUsecaseImpl,
        [DbFolderRepository, DbPlaylistRepository, DbSongSyncRepository]
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
            release_date: Some(NaiveDate::from_ymd(2013, 7, 14)),
            memo: Some("メモ".to_owned()),
            lyrics: Some("歌詞".to_owned()),
            artworks: vec![SongArtwork {
                picture: Rc::new(Picture {
                    bytes: vec![1, 2, 3, 4],
                    mime_type: "image/jpeg".to_owned(),
                }),
                picture_type: 3,
                description: "説明".to_owned(),
            }],
        }
    }

    #[test]
    fn test_register_db_root_folder() {
        fn song_path() -> LibSongPath {
            LibSongPath::new("song.flac")
        }
        fn entry_date() -> NaiveDate {
            NaiveDate::from_ymd(2021, 9, 21)
        }

        let mut mocks = Mocks::new();
        mocks.db_folder_repository(|m| {
            m.expect_register_not_exists().times(0);
        });
        mocks.db_song_sync_repository(|m| {
            m.expect_register()
                .times(1)
                .returning(|_, _, a_song_sync, a_folder_id, _| {
                    assert_eq!(a_folder_id, FolderIdMayRoot::Root);
                    assert_eq!(a_song_sync.title.as_deref(), Some("曲名"));
                    Ok(5)
                });
        });
        mocks.db_playlist_repository(|m| {
            m.expect_reset_listuped_flag()
                .times(1)
                .returning(|_| Ok(()));
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let mut s = song_sync();
            t.register_db(&tx, &song_path(), &mut s, entry_date())
                .unwrap();

            assert_eq!(s.title.as_deref(), Some("曲名"));
        });
    }
    #[test]
    fn test_register_db_no_title() {
        fn song_path() -> LibSongPath {
            LibSongPath::new("test/hoge/fuga.mp3")
        }
        fn entry_date() -> NaiveDate {
            NaiveDate::from_ymd(2021, 9, 21)
        }

        let mut mocks = Mocks::new();
        mocks.db_folder_repository(|m| {
            m.expect_register_not_exists()
                .times(1)
                .returning(|_, a_path| {
                    assert_eq!(a_path, &LibDirPath::new("test/hoge"));
                    Ok(FolderIdMayRoot::Folder(15))
                });
        });
        mocks.db_song_sync_repository(|m| {
            m.expect_register()
                .times(1)
                .returning(|_, _, a_song_sync, a_folder_id, _| {
                    assert_eq!(a_folder_id, FolderIdMayRoot::Folder(15));
                    assert_eq!(a_song_sync.title.as_deref(), Some("fuga"));
                    Ok(5)
                });
        });
        mocks.db_playlist_repository(|m| {
            m.expect_reset_listuped_flag()
                .times(1)
                .returning(|_| Ok(()));
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let mut s = song_sync();
            s.title = None;

            t.register_db(&tx, &song_path(), &mut s, entry_date())
                .unwrap();

            assert_eq!(s.title.as_deref(), Some("fuga"));
        });
    }
}
