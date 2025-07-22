use std::sync::Arc;

use anyhow::Result;
use murack_core_app::{
    Config,
    command::{
        CommandAdd, CommandArtworkGet, CommandCheck, CommandHelp, CommandMove, CommandPlaylist,
        CommandRemove, ResolveDapImpl, ResolveDataMatchImpl, ResolveExistanceImpl,
    },
    cui::Cui,
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

pub struct Registry<CUI>
where
    CUI: Cui + Send + Sync,
{
    cui: CUI,
    config: Config,
    db_registry: DbComponents,
}

impl<CUI> Registry<CUI>
where
    CUI: Cui + Send + Sync,
{
    pub fn new(cui: CUI, config: Config) -> Self {
        Self {
            cui,
            config,
            db_registry: DbComponents::new(),
        }
    }

    // -----------------------------
    // Commands

    pub fn command_add(self, command_line: &[String]) -> Result<TypeCommandAdd<CUI>> {
        let file_library_repository = self.file_library_repository();
        let sync_usecase = self.sync_usecase();
        CommandAdd::new(
            command_line,
            self.config,
            self.cui,
            file_library_repository,
            sync_usecase,
        )
    }

    pub fn command_check(self, command_line: &[String]) -> Result<TypeCommandCheck<CUI>> {
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

        let config = Arc::new(self.config);
        let cui = Arc::new(self.cui);

        CommandCheck::new(
            command_line,
            config.clone(),
            ResolveExistanceImpl::new(
                config.clone(),
                cui.clone(),
                file_library_repository1,
                song_usecase,
                sync_usecase,
                db_song_sync_repository1,
            ),
            ResolveDataMatchImpl::new(
                config.clone(),
                cui.clone(),
                file_library_repository2,
                check_usecase1,
                self.db_registry.db_artwork_repository(),
                db_song_sync_repository2,
            ),
            ResolveDapImpl::new(
                config,
                cui.clone(),
                file_library_repository3,
                check_usecase2,
            ),
            cui,
            file_library_repository4,
            check_usecase3,
            self.db_registry.db_song_repository(),
        )
    }

    pub fn command_move(self, command_line: &[String]) -> Result<TypeCommandMove> {
        let file_library_repository = self.file_library_repository();
        let song_usecase = self.song_usecase();
        CommandMove::new(
            command_line,
            self.config,
            file_library_repository,
            self.db_registry.db_song_repository(),
            self.db_registry.db_folder_repository(),
            song_usecase,
        )
    }

    pub fn command_remove(self, command_line: &[String]) -> Result<TypeCommandRemove<CUI>> {
        let song_usecase = self.song_usecase();
        CommandRemove::new(command_line, self.config, self.cui, song_usecase)
    }

    pub fn command_playlist(self) -> TypeCommandPlaylist<CUI> {
        let dap_playlist_usecase = self.dap_playlist_usecase();
        CommandPlaylist::new(self.config, self.cui, dap_playlist_usecase)
    }

    pub fn command_artwork_get(
        self,
        command_line: &[String],
    ) -> Result<TypeCommandArtworkGet<CUI>> {
        let file_library_repository = self.file_library_repository();
        CommandArtworkGet::new(command_line, self.config, self.cui, file_library_repository)
    }

    pub fn command_help(self, command_line: &[String]) -> Result<CommandHelp<CUI>> {
        CommandHelp::new(command_line, self.cui)
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

pub type TypeCommandAdd<CUI> = CommandAdd<CUI, FileLibraryRepositoryImpl, TypeSyncUsecase>;
pub type TypeCommandCheck<CUI> = CommandCheck<
    CUI,
    ResolveExistanceImpl<
        CUI,
        FileLibraryRepositoryImpl,
        TypeSongUsecase,
        TypeSyncUsecase,
        TypeDbSongSyncRepository,
    >,
    ResolveDataMatchImpl<
        CUI,
        FileLibraryRepositoryImpl,
        TypeCheckUsecase,
        TypeDbArtworkRepository,
        TypeDbSongSyncRepository,
    >,
    ResolveDapImpl<CUI, FileLibraryRepositoryImpl, TypeCheckUsecase>,
    FileLibraryRepositoryImpl,
    TypeCheckUsecase,
    TypeDbSongRepository,
>;
pub type TypeCommandMove = CommandMove<
    FileLibraryRepositoryImpl,
    TypeDbSongRepository,
    TypeDbFolderRepository,
    TypeSongUsecase,
>;
pub type TypeCommandRemove<CUI> = CommandRemove<CUI, TypeSongUsecase>;
pub type TypeCommandPlaylist<CUI> = CommandPlaylist<CUI, TypeDapPlaylistUsecase>;
pub type TypeCommandArtworkGet<CUI> = CommandArtworkGet<CUI, FileLibraryRepositoryImpl>;

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
