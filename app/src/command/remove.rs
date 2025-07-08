use crate::{cui::Cui, AppComponents, Config, Error};
use anyhow::Result;
use domain::{db_wrapper::ConnectionFactory, path::LibPathStr, song::SongUsecase};
use std::rc::Rc;

/// removeコマンド
///
/// ライブラリから曲を削除
pub struct CommandRemove {
    args: Args,

    config: Rc<Config>,
    cui: Rc<dyn Cui>,
    connection_factory: Rc<ConnectionFactory>,
    song_usecase: Rc<dyn SongUsecase>,
}

impl CommandRemove {
    pub fn new(command_line: &[String], app_components: &impl AppComponents) -> Result<Self> {
        Ok(Self {
            args: parse_args(command_line)?,
            config: app_components.config().clone(),
            cui: app_components.cui().clone(),
            connection_factory: app_components.connection_factory().clone(),
            song_usecase: app_components.song_usecase().clone(),
        })
    }

    /// このコマンドを実行
    /// # Arguments
    /// - command_line: コマンドライン引数
    pub fn run(&self) -> Result<()> {
        self.remove_db()?;
        self.remove_dap()?;
        self.remove_pc()
    }

    /// DBから削除
    pub fn remove_db(&self) -> Result<()> {
        let mut db = self.connection_factory.open()?;

        let song_path_list =
            db.run_in_transaction(|tx| self.song_usecase.delete_path_str_db(tx, &self.args.path))?;

        if song_path_list.is_empty() {
            self.cui.err(format_args!(
                "{}\n",
                domain::Error::DbPathStrNotFound(self.args.path.clone()),
            ));
            return Ok(());
        }

        cui_outln!(self.cui, "以下の曲をDBから削除しました。");
        for path in song_path_list {
            cui_outln!(self.cui, "{}", path);
        }
        cui_outln!(self.cui);

        Ok(())
    }

    /// DAPから削除
    pub fn remove_dap(&self) -> Result<()> {
        cui_outln!(self.cui, "DAPからの削除中...");

        match self
            .song_usecase
            .delete_path_str_dap(&self.config.dap_lib, &self.args.path)
        {
            Ok(_) => Ok(()),
            Err(e) => match e.downcast_ref() {
                //パスが見つからないエラーなら、出力してこの関数はOK
                Some(domain::Error::FilePathStrNotFound { .. }) => {
                    self.cui.err(format_args!("{}\n", e));
                    Ok(())
                }
                _ => Err(e),
            },
        }
    }

    /// PCから削除
    pub fn remove_pc(&self) -> Result<()> {
        cui_outln!(self.cui, "PCからの削除中...");

        match self
            .song_usecase
            .delete_path_str_pc(&self.config.pc_lib, &self.args.path)
        {
            Ok(_) => Ok(()),
            Err(e) => match e.downcast_ref() {
                //パスが見つからないエラーなら、出力してこの関数はOK
                Some(domain::Error::FilePathStrNotFound { .. }) => {
                    self.cui.err(format_args!("{}\n", e));
                    Ok(())
                }
                _ => Err(e),
            },
        }
    }
}

/// コマンドの引数
struct Args {
    /// 削除対象のパス
    ///
    /// ディレクトリ指定可
    path: LibPathStr,
}

/// コマンドの引数を解析
fn parse_args(command_line: &[String]) -> Result<Args> {
    match command_line {
        [s, ..] => Ok(Args {
            path: s.clone().into(),
        }),
        [] => Err(Error::InvalidCommandArgument {
            msg: "target path is not specified.".to_owned(),
        }
        .into()),
    }
}
