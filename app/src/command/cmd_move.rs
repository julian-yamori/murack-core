use anyhow::Result;
use murack_core_domain::{
    Error as DomainError, FileLibraryRepository,
    db::DbTransaction,
    folder::DbFolderRepository,
    path::LibPathStr,
    song::{DbSongRepository, SongUsecase},
};
use sqlx::PgPool;

use crate::{Config, Error};

/// moveコマンド
///
/// ライブラリ内で曲パスを移動
pub struct CommandMove<'config, FR, DSR, DFR, SS>
where
    FR: FileLibraryRepository,
    DSR: DbSongRepository,
    DFR: DbFolderRepository,
    SS: SongUsecase,
{
    args: CommandMoveArgs,

    config: &'config Config,
    file_library_repository: FR,
    db_song_repository: DSR,
    db_folder_repository: DFR,
    song_usecase: SS,
}

impl<'config, FR, DSR, DFR, SS> CommandMove<'config, FR, DSR, DFR, SS>
where
    FR: FileLibraryRepository,
    DSR: DbSongRepository,
    DFR: DbFolderRepository,
    SS: SongUsecase,
{
    pub fn new(
        args: CommandMoveArgs,
        config: &'config Config,
        file_library_repository: FR,
        db_song_repository: DSR,
        db_folder_repository: DFR,
        song_usecase: SS,
    ) -> Self {
        Self {
            args,
            config,
            file_library_repository,
            db_folder_repository,
            db_song_repository,
            song_usecase,
        }
    }

    /// このコマンドを実行
    pub async fn run(&self, db_pool: &PgPool) -> Result<()> {
        self.check_pc_exist()?;
        self.check_dap_exist()?;
        self.check_db_exist(db_pool).await?;

        let src_path_str = &self.args.src_path;
        let dest_path_str = &self.args.dest_path;

        //PC内で移動
        self.file_library_repository.move_path_str(
            &self.config.pc_lib,
            src_path_str,
            dest_path_str,
        )?;

        //DAP内で移動
        self.file_library_repository.move_path_str(
            &self.config.dap_lib,
            src_path_str,
            dest_path_str,
        )?;

        //DB内で移動
        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };
        self.song_usecase
            .move_path_str_db(&mut tx, src_path_str, dest_path_str)
            .await?;
        tx.commit().await?;

        Ok(())
    }

    /// PCの移動先に既に存在しないか確認する
    fn check_pc_exist(&self) -> Result<()> {
        let pc_lib = &self.config.pc_lib;
        let dest_path_str = &self.args.dest_path;

        if self
            .file_library_repository
            .is_exist_path_str(pc_lib, dest_path_str)?
        {
            return Err(DomainError::FilePathStrAlreadyExists {
                lib_root: pc_lib.clone(),
                path_str: dest_path_str.clone(),
            }
            .into());
        }

        Ok(())
    }
    /// DAPの移動先に既に存在しないか確認する
    fn check_dap_exist(&self) -> Result<()> {
        let dap_lib = &self.config.dap_lib;
        let dest_path_str = &self.args.dest_path;

        if self
            .file_library_repository
            .is_exist_path_str(dap_lib, dest_path_str)?
        {
            return Err(DomainError::FilePathStrAlreadyExists {
                lib_root: dap_lib.clone(),
                path_str: dest_path_str.clone(),
            }
            .into());
        }

        Ok(())
    }

    /// DBの移動先に既に存在しないか確認する
    async fn check_db_exist(&self, db_pool: &PgPool) -> Result<()> {
        let dest_path_str = &self.args.dest_path;

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        //曲のチェック
        let dest_song_path = dest_path_str.to_song_path();
        if self
            .db_song_repository
            .is_exist_path(&mut tx, &dest_song_path)
            .await?
        {
            return Err(DomainError::DbSongAlreadyExists(dest_song_path).into());
        }

        //フォルダのチェック
        let dest_dir_path = dest_path_str.to_dir_path();
        if self
            .db_folder_repository
            .is_exist_path(&mut tx, &dest_dir_path)
            .await?
        {
            return Err(DomainError::DbFolderAlreadyExists(dest_dir_path).into());
        }

        Ok(())
    }
}

/// コマンドの引数
pub struct CommandMoveArgs {
    /// 移動元のパス
    ///
    /// ディレクトリ指定可(dest_pathもディレクトリである必要あり)
    pub src_path: LibPathStr,

    /// 移動先のパス
    ///
    /// ディレクトリ指定可(src_pathもディレクトリであること)
    pub dest_path: LibPathStr,
}

impl CommandMoveArgs {
    /// コマンドの引数を解析
    pub fn parse(command_line: &[String]) -> Result<CommandMoveArgs> {
        match command_line {
            [src, dest, ..] => Ok(CommandMoveArgs {
                src_path: src.clone().into(),
                dest_path: dest.clone().into(),
            }),
            [_] => Err(Error::InvalidCommandArgument {
                msg: "destination path is not specified.".to_owned(),
            }
            .into()),
            [] => Err(Error::InvalidCommandArgument {
                msg: "source file path is not specified.".to_owned(),
            }
            .into()),
        }
    }
}
