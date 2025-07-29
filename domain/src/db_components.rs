use crate::{
    folder::DbFolderRepositoryImpl,
    playlist::{DbPlaylistRepositoryImpl, DbPlaylistTrackRepositoryImpl},
    sync::DbTrackSyncRepositoryImpl,
    tag::DbTrackTagRepositoryImpl,
    track::DbTrackRepositoryImpl,
};

/// data層DB機能のDIを解決するオブジェクト
pub struct DbComponents {}

impl DbComponents {
    pub fn new() -> Self {
        Self {}
    }

    pub fn db_folder_repository(&self) -> TypeDbFolderRepository {
        DbFolderRepositoryImpl::new()
    }

    pub fn db_playlist_repository(&self) -> TypeDbPlaylistRepository {
        DbPlaylistRepositoryImpl::new()
    }

    pub fn db_playlist_track_repository(&self) -> TypeDbPlaylistTrackRepository {
        DbPlaylistTrackRepositoryImpl::new()
    }

    pub fn db_track_repository(&self) -> TypeDbTrackRepository {
        DbTrackRepositoryImpl::new()
    }

    pub fn db_track_sync_repository(&self) -> TypeDbTrackSyncRepository {
        DbTrackSyncRepositoryImpl::new()
    }

    pub fn db_track_tag_repository(&self) -> TypeDbTrackTagRepository {
        DbTrackTagRepositoryImpl::new()
    }
}

impl Default for DbComponents {
    fn default() -> Self {
        Self::new()
    }
}

pub type TypeDbFolderRepository = DbFolderRepositoryImpl;
pub type TypeDbPlaylistRepository = DbPlaylistRepositoryImpl;
pub type TypeDbPlaylistTrackRepository = DbPlaylistTrackRepositoryImpl;
pub type TypeDbTrackRepository = DbTrackRepositoryImpl;
pub type TypeDbTrackSyncRepository = DbTrackSyncRepositoryImpl;
pub type TypeDbTrackTagRepository = DbTrackTagRepositoryImpl;
