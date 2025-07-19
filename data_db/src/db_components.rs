use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use crate::{
    artwork::{
        ArtworkCache, ArtworkDao, ArtworkImageDaoImpl, DbArtworkRepositoryImpl, SongArtworkDaoImpl,
    },
    folder::{DbFolderRepositoryImpl, FolderPathDaoImpl},
    playlist::{
        DbPlaylistRepositoryImpl, DbPlaylistSongRepositoryImpl, PlaylistDaoImpl,
        PlaylistSongDaoImpl,
    },
    song::{DbSongRepositoryImpl, DbSongSyncRepositoryImpl, SongDaoImpl, SongSyncDaoImpl},
    song_lister::{SongFinderImpl, SongListerFilterImpl},
    tag::{DbSongTagRepositoryImpl, SongTagsDaoImpl},
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
        SongFinderImpl::new(
            PlaylistDaoImpl {},
            PlaylistSongDaoImpl {},
            SongListerFilterImpl {},
        )
    }

    pub fn db_artwork_repository(&self) -> TypeDbArtworkRepository {
        DbArtworkRepositoryImpl::new(
            self.artwork_cache.clone(),
            ArtworkDao {},
            ArtworkImageDaoImpl {},
            SongArtworkDaoImpl {},
        )
    }

    pub fn db_folder_repository(&self) -> TypeDbFolderRepository {
        DbFolderRepositoryImpl::new(FolderPathDaoImpl {})
    }

    pub fn db_playlist_repository(&self) -> TypeDbPlaylistRepository {
        DbPlaylistRepositoryImpl::new(PlaylistDaoImpl {})
    }

    pub fn db_playlist_song_repository(&self) -> TypeDbPlaylistSongRepository {
        DbPlaylistSongRepositoryImpl::new(PlaylistDaoImpl {}, PlaylistSongDaoImpl {})
    }

    pub fn db_song_repository(&self) -> TypeDbSongRepository {
        DbSongRepositoryImpl::new(SongDaoImpl {})
    }

    pub fn db_song_sync_repository(&self) -> TypeDbSongSyncRepository {
        DbSongSyncRepositoryImpl::new(
            self.db_artwork_repository(),
            SongDaoImpl {},
            SongSyncDaoImpl {},
        )
    }

    pub fn db_song_tag_repository(&self) -> TypeDbSongTagRepository {
        DbSongTagRepositoryImpl::new(SongTagsDaoImpl {})
    }
}

impl Default for DbComponents {
    fn default() -> Self {
        Self::new()
    }
}

pub type TypeSongFinder =
    SongFinderImpl<PlaylistDaoImpl, PlaylistSongDaoImpl, SongListerFilterImpl>;
pub type TypeDbArtworkRepository = DbArtworkRepositoryImpl<ArtworkImageDaoImpl, SongArtworkDaoImpl>;
pub type TypeDbFolderRepository = DbFolderRepositoryImpl<FolderPathDaoImpl>;
pub type TypeDbPlaylistRepository = DbPlaylistRepositoryImpl<PlaylistDaoImpl>;
pub type TypeDbPlaylistSongRepository =
    DbPlaylistSongRepositoryImpl<PlaylistDaoImpl, PlaylistSongDaoImpl>;
pub type TypeDbSongRepository = DbSongRepositoryImpl<SongDaoImpl>;
pub type TypeDbSongSyncRepository =
    DbSongSyncRepositoryImpl<TypeDbArtworkRepository, SongDaoImpl, SongSyncDaoImpl>;
pub type TypeDbSongTagRepository = DbSongTagRepositoryImpl<SongTagsDaoImpl>;
