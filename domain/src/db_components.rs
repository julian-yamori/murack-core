use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use crate::{
    artwork::{ArtworkCache, DbArtworkRepositoryImpl},
    folder::DbFolderRepositoryImpl,
    playlist::{DbPlaylistRepositoryImpl, DbPlaylistTrackRepositoryImpl},
    sync::DbTrackSyncRepositoryImpl,
    tag::DbTrackTagRepositoryImpl,
    track::DbTrackRepositoryImpl,
};

/// data層DB機能のDIを解決するオブジェクト
pub struct DbComponents {
    artwork_cache: Lazy<Arc<Mutex<ArtworkCache>>>,
}

impl DbComponents {
    pub fn new() -> Self {
        Self {
            artwork_cache: Lazy::new(|| Arc::new(Mutex::new(ArtworkCache::new()))),
        }
    }

    pub fn db_artwork_repository(&self) -> TypeDbArtworkRepository {
        DbArtworkRepositoryImpl::new(self.artwork_cache.clone())
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
        DbTrackSyncRepositoryImpl::new(self.db_artwork_repository())
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

pub type TypeDbArtworkRepository = DbArtworkRepositoryImpl;
pub type TypeDbFolderRepository = DbFolderRepositoryImpl;
pub type TypeDbPlaylistRepository = DbPlaylistRepositoryImpl;
pub type TypeDbPlaylistTrackRepository = DbPlaylistTrackRepositoryImpl;
pub type TypeDbTrackRepository = DbTrackRepositoryImpl;
pub type TypeDbTrackSyncRepository = DbTrackSyncRepositoryImpl<TypeDbArtworkRepository>;
pub type TypeDbTrackTagRepository = DbTrackTagRepositoryImpl;
