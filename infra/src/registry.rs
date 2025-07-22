use anyhow::Result;
use murack_core_app::{
    Config,
    command::{
        CommandAdd, CommandArtworkGet, CommandCheck, CommandHelp, CommandMove, CommandPlaylist,
        CommandRemove, ResolveDapImpl, ResolveDataMatchImpl, ResolveExistanceImpl,
    },
    cui::StdCui,
};
use murack_core_data_db::db_components::{
    DbComponents, TypeDbArtworkRepository, TypeDbFolderRepository, TypeDbPlaylistRepository,
    TypeDbPlaylistSongRepository, TypeDbSongRepository, TypeDbSongSyncRepository,
    TypeDbSongTagRepository, TypeSongFinder,
};
use murack_core_data_file::{DapRepositoryImpl, FileLibraryRepositoryImpl};
use murack_core_domain::{
    check::CheckUsecaseImpl, dap::DapPlaylistUsecaseImpl, folder::FolderUsecaseImpl,
    song::SongUsecaseImpl, sync::SyncUsecaseImpl,
};

pub struct Registry {
    cui: StdCui,
    config: Config,
    db_registry: DbComponents,
}

impl Registry {
    pub fn new(cui: StdCui, config: Config) -> Self {
        Self {
            cui,
            config,
            db_registry: DbComponents::new(),
        }
    }

    // -----------------------------
    // Commands

    pub fn command_add(&self, command_line: &[String]) -> Result<TypeCommandAdd> {
        let file_library_repository = self.file_library_repository();
        let sync_usecase = self.sync_usecase();
        CommandAdd::new(
            command_line,
            &self.config,
            &self.cui,
            file_library_repository,
            sync_usecase,
        )
    }

    pub fn command_check(&self, command_line: &[String]) -> Result<TypeCommandCheck> {
        let file_library_repository1 = self.file_library_repository();
        let file_library_repository2 = self.file_library_repository();
        let file_library_repository3 = self.file_library_repository();
        let file_library_repository4 = self.file_library_repository();
        let song_usecase = self.song_usecase();
        let sync_usecase = self.sync_usecase();
        let check_usecase1 = self.check_usecase();
        let check_usecase2 = self.check_usecase();
        let check_usecase3 = self.check_usecase();
        let db_song_sync_repository1 = self.db_registry.db_song_sync_repository();
        let db_song_sync_repository2 = self.db_registry.db_song_sync_repository();

        CommandCheck::new(
            command_line,
            &self.config,
            ResolveExistanceImpl::new(
                &self.config,
                &self.cui,
                file_library_repository1,
                song_usecase,
                sync_usecase,
                db_song_sync_repository1,
            ),
            ResolveDataMatchImpl::new(
                &self.config,
                &self.cui,
                file_library_repository2,
                check_usecase1,
                self.db_registry.db_artwork_repository(),
                db_song_sync_repository2,
            ),
            ResolveDapImpl::new(
                &self.config,
                &self.cui,
                file_library_repository3,
                check_usecase2,
            ),
            &self.cui,
            file_library_repository4,
            check_usecase3,
            self.db_registry.db_song_repository(),
        )
    }

    pub fn command_move(&self, command_line: &[String]) -> Result<TypeCommandMove> {
        let file_library_repository = self.file_library_repository();
        let song_usecase = self.song_usecase();
        CommandMove::new(
            command_line,
            &self.config,
            file_library_repository,
            self.db_registry.db_song_repository(),
            self.db_registry.db_folder_repository(),
            song_usecase,
        )
    }

    pub fn command_remove(&self, command_line: &[String]) -> Result<TypeCommandRemove> {
        let song_usecase = self.song_usecase();
        CommandRemove::new(command_line, &self.config, &self.cui, song_usecase)
    }

    pub fn command_playlist(&self) -> TypeCommandPlaylist {
        let dap_playlist_usecase = self.dap_playlist_usecase();
        CommandPlaylist::new(&self.config, &self.cui, dap_playlist_usecase)
    }

    pub fn command_artwork_get(&self, command_line: &[String]) -> Result<TypeCommandArtworkGet> {
        let file_library_repository = self.file_library_repository();
        CommandArtworkGet::new(
            command_line,
            &self.config,
            &self.cui,
            file_library_repository,
        )
    }

    pub fn command_help(&self, command_line: &[String]) -> Result<CommandHelp<StdCui>> {
        CommandHelp::new(command_line, &self.cui)
    }

    // -----------------------------
    // Domain Services

    fn check_usecase(&self) -> TypeCheckUsecase {
        CheckUsecaseImpl::new(
            self.db_registry.db_song_sync_repository(),
            self.file_library_repository(),
        )
    }

    fn dap_playlist_usecase(&self) -> TypeDapPlaylistUsecase {
        DapPlaylistUsecaseImpl::new(
            self.dap_repository(),
            self.db_registry.db_playlist_repository(),
            self.db_registry.song_finder(),
        )
    }

    fn folder_usecase(&self) -> TypeFolderUsecase {
        FolderUsecaseImpl::new(
            self.db_registry.db_folder_repository(),
            self.db_registry.db_song_repository(),
        )
    }

    fn song_usecase(&self) -> TypeSongUsecase {
        SongUsecaseImpl::new(
            self.file_library_repository(),
            self.db_registry.db_artwork_repository(),
            self.db_registry.db_folder_repository(),
            self.db_registry.db_playlist_repository(),
            self.db_registry.db_playlist_song_repository(),
            self.db_registry.db_song_repository(),
            self.db_registry.db_song_tag_repository(),
            self.folder_usecase(),
        )
    }

    fn sync_usecase(&self) -> TypeSyncUsecase {
        SyncUsecaseImpl::new(
            self.db_registry.db_folder_repository(),
            self.db_registry.db_playlist_repository(),
            self.db_registry.db_song_sync_repository(),
        )
    }

    // -----------------------------
    // Repositories

    fn file_library_repository(&self) -> FileLibraryRepositoryImpl {
        FileLibraryRepositoryImpl {}
    }

    fn dap_repository(&self) -> DapRepositoryImpl {
        DapRepositoryImpl {}
    }
}

pub type TypeCommandAdd<'config, 'cui> =
    CommandAdd<'config, 'cui, StdCui, FileLibraryRepositoryImpl, TypeSyncUsecase>;
pub type TypeCommandCheck<'config, 'cui> = CommandCheck<
    'config,
    'cui,
    StdCui,
    ResolveExistanceImpl<
        'config,
        'cui,
        StdCui,
        FileLibraryRepositoryImpl,
        TypeSongUsecase,
        TypeSyncUsecase,
        TypeDbSongSyncRepository,
    >,
    ResolveDataMatchImpl<
        'config,
        'cui,
        StdCui,
        FileLibraryRepositoryImpl,
        TypeCheckUsecase,
        TypeDbArtworkRepository,
        TypeDbSongSyncRepository,
    >,
    ResolveDapImpl<'config, 'cui, StdCui, FileLibraryRepositoryImpl, TypeCheckUsecase>,
    FileLibraryRepositoryImpl,
    TypeCheckUsecase,
    TypeDbSongRepository,
>;
pub type TypeCommandMove<'config> = CommandMove<
    'config,
    FileLibraryRepositoryImpl,
    TypeDbSongRepository,
    TypeDbFolderRepository,
    TypeSongUsecase,
>;
pub type TypeCommandRemove<'config, 'cui> = CommandRemove<'config, 'cui, StdCui, TypeSongUsecase>;
pub type TypeCommandPlaylist<'config, 'cui> =
    CommandPlaylist<'config, 'cui, StdCui, TypeDapPlaylistUsecase>;
pub type TypeCommandArtworkGet<'config, 'cui> =
    CommandArtworkGet<'config, 'cui, StdCui, FileLibraryRepositoryImpl>;

type TypeCheckUsecase = CheckUsecaseImpl<TypeDbSongSyncRepository, FileLibraryRepositoryImpl>;
type TypeDapPlaylistUsecase =
    DapPlaylistUsecaseImpl<DapRepositoryImpl, TypeDbPlaylistRepository, TypeSongFinder>;
type TypeFolderUsecase = FolderUsecaseImpl<TypeDbFolderRepository, TypeDbSongRepository>;
type TypeSongUsecase = SongUsecaseImpl<
    FileLibraryRepositoryImpl,
    TypeDbArtworkRepository,
    TypeDbFolderRepository,
    TypeDbPlaylistRepository,
    TypeDbPlaylistSongRepository,
    TypeDbSongRepository,
    TypeDbSongTagRepository,
    TypeFolderUsecase,
>;
type TypeSyncUsecase =
    SyncUsecaseImpl<TypeDbFolderRepository, TypeDbPlaylistRepository, TypeDbSongSyncRepository>;
