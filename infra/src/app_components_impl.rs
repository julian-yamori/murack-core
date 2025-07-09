use paste::paste;
use std::rc::Rc;
use walk_base_2_app::{AppComponents, Config, cui::Cui};
use walk_base_2_data_db::DbComponents;
use walk_base_2_data_file::{DapRepositoryImpl, FileLibraryRepositoryImpl};
use walk_base_2_domain::{
    FileLibraryRepository,
    artwork::DbArtworkRepository,
    check::{CheckUsecase, CheckUsecaseImpl},
    dap::{DapPlaylistUsecase, DapPlaylistUsecaseImpl},
    db_wrapper::ConnectionFactory,
    folder::{DbFolderRepository, FolderUsecase, FolderUsecaseImpl},
    playlist::DbPlaylistRepository,
    song::{DbSongRepository, SongUsecase, SongUsecaseImpl},
    sync::{DbSongSyncRepository, SyncUsecase, SyncUsecaseImpl},
};

macro_rules! struct_define {
    ($($t: ident),*) => {
        paste! {
            /// app層のDIを解決するオブジェクトの実装
            #[derive(Getters)]
            pub struct AppComponentsImpl {
                db_components: Rc<DbComponents>,
                config: Rc<Config>,
                connection_factory: Rc<ConnectionFactory>,
                $(
                    [<$t:snake>]: Rc<dyn $t>,
                )*
            }
        }
    };
}
struct_define![
    Cui,
    FileLibraryRepository,
    CheckUsecase,
    DapPlaylistUsecase,
    FolderUsecase,
    SongUsecase,
    SyncUsecase
];

impl AppComponentsImpl {
    pub fn new(cui: Rc<dyn Cui>, config: Config) -> Self {
        let db_components = Rc::new(DbComponents::new());
        let file_library_repository = Rc::new(FileLibraryRepositoryImpl {});
        let dap_repository = Rc::new(DapRepositoryImpl {});

        let folder_usecase = Rc::new(FolderUsecaseImpl::new(
            db_components.db_folder_repository().clone(),
            db_components.db_song_repository().clone(),
        ));

        Self {
            connection_factory: Rc::new(ConnectionFactory::File(config.db_path.clone())),
            config: Rc::new(config),
            check_usecase: Rc::new(CheckUsecaseImpl::new(
                db_components.db_song_sync_repository().clone(),
                file_library_repository.clone(),
            )),
            dap_playlist_usecase: Rc::new(DapPlaylistUsecaseImpl::new(
                dap_repository,
                db_components.db_playlist_repository().clone(),
                db_components.song_finder().clone(),
            )),
            song_usecase: Rc::new(SongUsecaseImpl::new(
                file_library_repository.clone(),
                db_components.db_artwork_repository().clone(),
                db_components.db_folder_repository().clone(),
                db_components.db_playlist_repository().clone(),
                db_components.db_playlist_song_repository().clone(),
                db_components.db_song_repository().clone(),
                db_components.db_song_tag_repository().clone(),
                folder_usecase.clone(),
            )),
            sync_usecase: Rc::new(SyncUsecaseImpl::new(
                db_components.db_folder_repository().clone(),
                db_components.db_playlist_repository().clone(),
                db_components.db_song_sync_repository().clone(),
            )),
            folder_usecase,
            cui,
            file_library_repository,
            db_components,
        }
    }
}

macro_rules! getter {
    ($t: ident) => {
        paste! {
            fn [<$t:snake>](&self) -> &Rc<dyn $t> {
                &self.[<$t:snake>]
            }
        }
    };
}
macro_rules! repos_getter {
    ($t: ident) => {
        paste! {
            fn [<$t:snake>](&self) -> &Rc<dyn $t> {
                self.db_components.[<$t:snake>]()
            }
        }
    };
}

impl AppComponents for AppComponentsImpl {
    fn config(&self) -> &Rc<Config> {
        &self.config
    }
    fn connection_factory(&self) -> &Rc<ConnectionFactory> {
        &self.connection_factory
    }

    getter!(Cui);
    getter!(FileLibraryRepository);

    getter!(CheckUsecase);
    getter!(DapPlaylistUsecase);
    getter!(FolderUsecase);
    getter!(SongUsecase);
    getter!(SyncUsecase);

    repos_getter!(DbArtworkRepository);
    repos_getter!(DbFolderRepository);
    repos_getter!(DbPlaylistRepository);
    repos_getter!(DbSongRepository);
    repos_getter!(DbSongSyncRepository);
}
