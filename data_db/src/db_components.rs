use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use crate::{
    artwork::{ArtworkCache, DbArtworkRepositoryImpl},
    folder::DbFolderRepositoryImpl,
    playlist::{DbPlaylistRepositoryImpl, DbPlaylistSongRepositoryImpl},
    song::{DbSongRepositoryImpl, DbSongSyncRepositoryImpl},
    song_lister::{SongFinderImpl, SongListerFilterImpl},
    tag::DbSongTagRepositoryImpl,
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

    pub fn song_finder(&self) -> TypeSongFinder {
        SongFinderImpl::new(SongListerFilterImpl {})
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

    pub fn db_playlist_song_repository(&self) -> TypeDbPlaylistSongRepository {
        DbPlaylistSongRepositoryImpl::new()
    }

    pub fn db_song_repository(&self) -> TypeDbSongRepository {
        DbSongRepositoryImpl::new()
    }

    pub fn db_song_sync_repository(&self) -> TypeDbSongSyncRepository {
        DbSongSyncRepositoryImpl::new(self.db_artwork_repository())
    }

    pub fn db_song_tag_repository(&self) -> TypeDbSongTagRepository {
        DbSongTagRepositoryImpl::new()
    }
}

impl Default for DbComponents {
    fn default() -> Self {
        Self::new()
    }
}

pub type TypeSongFinder = SongFinderImpl<SongListerFilterImpl>;
pub type TypeDbArtworkRepository = DbArtworkRepositoryImpl;
pub type TypeDbFolderRepository = DbFolderRepositoryImpl;
pub type TypeDbPlaylistRepository = DbPlaylistRepositoryImpl;
pub type TypeDbPlaylistSongRepository = DbPlaylistSongRepositoryImpl;
pub type TypeDbSongRepository = DbSongRepositoryImpl;
pub type TypeDbSongSyncRepository = DbSongSyncRepositoryImpl<TypeDbArtworkRepository>;
pub type TypeDbSongTagRepository = DbSongTagRepositoryImpl;
