use super::{
    artwork::{
        ArtworkCache, ArtworkDaoImpl, ArtworkImageDaoImpl, DbArtworkRepositoryImpl,
        SongArtworkDaoImpl,
    },
    filter::{DbFilterRepositoryImpl, FilterDaoImpl},
    folder::{DbFolderRepositoryImpl, FolderPathDaoImpl},
    playlist::{
        DbPlaylistRepositoryImpl, DbPlaylistSongRepositoryImpl, PlaylistDaoImpl,
        PlaylistSongDaoImpl,
    },
    song::{DbSongRepositoryImpl, DbSongSyncRepositoryImpl, SongDaoImpl, SongSyncDaoImpl},
    song_lister::{SongFinderImpl, SongListerFilterImpl},
    tag::{DbSongTagRepositoryImpl, SongTagsDaoImpl},
};
use domain::{
    artwork::DbArtworkRepository,
    dap::SongFinder,
    folder::DbFolderRepository,
    playlist::{DbPlaylistRepository, DbPlaylistSongRepository},
    song::DbSongRepository,
    sync::DbSongSyncRepository,
    tag::DbSongTagRepository,
};
use paste::paste;
use std::{cell::RefCell, rc::Rc};

macro_rules! struct_define {
    ($($t: ident),*) => {
        paste! {
            /// data層DB機能のDIを解決するオブジェクト
            #[derive(Getters)]
            pub struct DbComponents {
                $(
                    [<$t:snake>]: Rc<dyn $t>,
                )*
            }
        }
    };
}
struct_define![
    DbArtworkRepository,
    DbFolderRepository,
    DbPlaylistRepository,
    DbPlaylistSongRepository,
    DbSongRepository,
    DbSongSyncRepository,
    DbSongTagRepository,
    SongFinder
];

impl DbComponents {
    pub fn new() -> Self {
        let artwork_cache = Rc::new(RefCell::new(ArtworkCache::new()));

        let artwork_dao = Rc::new(ArtworkDaoImpl {});
        let artwork_image_dao = Rc::new(ArtworkImageDaoImpl {});
        let filter_dao = Rc::new(FilterDaoImpl {});
        let folder_path_dao = Rc::new(FolderPathDaoImpl {});
        let playlist_dao = Rc::new(PlaylistDaoImpl {});
        let playlist_song_dao = Rc::new(PlaylistSongDaoImpl {});
        let song_dao = Rc::new(SongDaoImpl {});
        let song_artwork_dao = Rc::new(SongArtworkDaoImpl {});
        let song_sync_dao = Rc::new(SongSyncDaoImpl {});
        let song_tags_dao = Rc::new(SongTagsDaoImpl {});

        let db_artwork_repository = Rc::new(DbArtworkRepositoryImpl::new(
            artwork_cache,
            artwork_dao,
            artwork_image_dao,
            song_artwork_dao,
        ));
        let db_filter_repository = Rc::new(DbFilterRepositoryImpl::new(filter_dao));

        let song_lister_filter = Rc::new(SongListerFilterImpl {});

        Self {
            song_finder: Rc::new(SongFinderImpl::new(
                playlist_dao.clone(),
                playlist_song_dao.clone(),
                db_filter_repository,
                song_lister_filter,
            )),
            db_folder_repository: Rc::new(DbFolderRepositoryImpl::new(folder_path_dao)),
            db_playlist_repository: Rc::new(DbPlaylistRepositoryImpl::new(playlist_dao.clone())),
            db_playlist_song_repository: Rc::new(DbPlaylistSongRepositoryImpl::new(
                playlist_dao,
                playlist_song_dao,
            )),
            db_song_repository: Rc::new(DbSongRepositoryImpl::new(song_dao.clone())),
            db_song_sync_repository: Rc::new(DbSongSyncRepositoryImpl::new(
                db_artwork_repository.clone(),
                song_dao,
                song_sync_dao,
            )),
            db_song_tag_repository: Rc::new(DbSongTagRepositoryImpl::new(song_tags_dao)),
            db_artwork_repository,
        }
    }
}

impl Default for DbComponents {
    fn default() -> Self {
        Self::new()
    }
}
