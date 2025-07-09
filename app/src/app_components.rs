use crate::{Config, cui::Cui};
use domain::{
    FileLibraryRepository,
    artwork::DbArtworkRepository,
    check::CheckUsecase,
    dap::DapPlaylistUsecase,
    db_wrapper::ConnectionFactory,
    folder::{DbFolderRepository, FolderUsecase},
    playlist::DbPlaylistRepository,
    song::{DbSongRepository, SongUsecase},
    sync::{DbSongSyncRepository, SyncUsecase},
};
use paste::paste;
use std::rc::Rc;

macro_rules! getter {
    ($t: ident) => {
        paste! {
            fn [<$t:snake>](&self) -> &Rc<dyn $t>;
        }
    };
}

/// app層のDIを解決するオブジェクト
pub trait AppComponents {
    fn config(&self) -> &Rc<Config>;
    fn connection_factory(&self) -> &Rc<ConnectionFactory>;

    getter!(Cui);
    getter!(FileLibraryRepository);
    getter!(DbArtworkRepository);
    getter!(DbFolderRepository);
    getter!(DbPlaylistRepository);
    getter!(DbSongRepository);
    getter!(DbSongSyncRepository);
    getter!(CheckUsecase);
    getter!(DapPlaylistUsecase);
    getter!(FolderUsecase);
    getter!(SongUsecase);
    getter!(SyncUsecase);
}
