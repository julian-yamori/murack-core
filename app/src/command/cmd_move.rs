use crate::{AppComponents, Config, Error};
use anyhow::Result;
use domain::{
    FileLibraryRepository,
    db_wrapper::{ConnectionFactory, ConnectionWrapper},
    folder::DbFolderRepository,
    path::LibPathStr,
    song::{DbSongRepository, SongUsecase},
};
use std::rc::Rc;

/// moveコマンド
///
/// ライブラリ内で曲パスを移動
pub struct CommandMove {
    args: Args,

    config: Rc<Config>,
    connection_factory: Rc<ConnectionFactory>,
    file_library_repository: Rc<dyn FileLibraryRepository>,
    db_song_repository: Rc<dyn DbSongRepository>,
    db_folder_repository: Rc<dyn DbFolderRepository>,
    song_usecase: Rc<dyn SongUsecase>,
}

impl CommandMove {
    pub fn new(command_line: &[String], app_components: &impl AppComponents) -> Result<Self> {
        Ok(Self {
            args: parse_args(command_line)?,
            config: app_components.config().clone(),
            connection_factory: app_components.connection_factory().clone(),
            file_library_repository: app_components.file_library_repository().clone(),
            db_folder_repository: app_components.db_folder_repository().clone(),
            db_song_repository: app_components.db_song_repository().clone(),
            song_usecase: app_components.song_usecase().clone(),
        })
    }

    /// このコマンドを実行
    pub fn run(&self) -> Result<()> {
        self.check_pc_exist()?;
        self.check_dap_exist()?;

        let mut db = self.connection_factory.open()?;
        self.check_db_exist(&mut db)?;

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
        db.run_in_transaction(|tx| {
            self.song_usecase
                .move_path_str_db(tx, src_path_str, dest_path_str)
        })
    }

    /// PCの移動先に既に存在しないか確認する
    fn check_pc_exist(&self) -> Result<()> {
        let pc_lib = &self.config.pc_lib;
        let dest_path_str = &self.args.dest_path;

        if self
            .file_library_repository
            .is_exist_path_str(pc_lib, dest_path_str)?
        {
            return Err(domain::Error::FilePathStrAlreadyExists {
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
            return Err(domain::Error::FilePathStrAlreadyExists {
                lib_root: dap_lib.clone(),
                path_str: dest_path_str.clone(),
            }
            .into());
        }

        Ok(())
    }

    /// DBの移動先に既に存在しないか確認する
    fn check_db_exist(&self, db: &mut ConnectionWrapper) -> Result<()> {
        let dest_path_str = &self.args.dest_path;

        db.run_in_transaction(|tx| {
            //曲のチェック
            let dest_song_path = dest_path_str.to_song_path();
            if self.db_song_repository.is_exist_path(tx, &dest_song_path)? {
                return Err(domain::Error::DbSongAlreadyExists(dest_song_path).into());
            }

            //フォルダのチェック
            let dest_dir_path = dest_path_str.to_dir_path();
            if self
                .db_folder_repository
                .is_exist_path(tx, &dest_dir_path)?
            {
                return Err(domain::Error::DbFolderAlreadyExists(dest_dir_path).into());
            }

            Ok(())
        })
    }
}

/// コマンドの引数
struct Args {
    /// 移動元のパス
    ///
    /// ディレクトリ指定可(dest_pathもディレクトリである必要あり)
    src_path: LibPathStr,

    /// 移動先のパス
    ///
    /// ディレクトリ指定可(src_pathもディレクトリであること)
    dest_path: LibPathStr,
}

/// コマンドの引数を解析
fn parse_args(command_line: &[String]) -> Result<Args> {
    match command_line {
        [src, dest, ..] => Ok(Args {
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
