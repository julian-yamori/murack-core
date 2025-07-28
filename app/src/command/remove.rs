use anyhow::Result;
use murack_core_domain::{Error as DomainError, NonEmptyString, track::TrackUsecase};
use sqlx::PgPool;

use crate::{Config, Error, cui::Cui};

/// removeコマンド
///
/// ライブラリから曲を削除
pub struct CommandRemove<'config, 'cui, CUI, SS>
where
    CUI: Cui,
    SS: TrackUsecase,
{
    args: CommandRemoveArgs,
    config: &'config Config,
    cui: &'cui CUI,
    track_usecase: SS,
}

impl<'config, 'cui, CUI, SS> CommandRemove<'config, 'cui, CUI, SS>
where
    CUI: Cui,
    SS: TrackUsecase,
{
    pub fn new(
        args: CommandRemoveArgs,
        config: &'config Config,
        cui: &'cui CUI,
        track_usecase: SS,
    ) -> Self {
        Self {
            args,
            config,
            cui,
            track_usecase,
        }
    }

    /// このコマンドを実行
    /// # Arguments
    /// - command_line: コマンドライン引数
    pub async fn run(&self, db_pool: &PgPool) -> Result<()> {
        self.remove_db(db_pool).await?;
        self.remove_dap()?;
        self.remove_pc()
    }

    /// DBから削除
    pub async fn remove_db(&self, db_pool: &PgPool) -> Result<()> {
        let mut tx = db_pool.begin().await?;
        let track_path_list = self
            .track_usecase
            .delete_path_str_db(&mut tx, &self.args.path)
            .await?;
        tx.commit().await?;

        if track_path_list.is_empty() {
            self.cui.err(format_args!(
                "{}\n",
                DomainError::DbPathStrNotFound(self.args.path.clone()),
            ))?;
            return Ok(());
        }

        cui_outln!(self.cui, "以下の曲をDBから削除しました。")?;
        for path in track_path_list {
            cui_outln!(self.cui, "{}", path)?;
        }
        cui_outln!(self.cui)?;

        Ok(())
    }

    /// DAPから削除
    pub fn remove_dap(&self) -> Result<()> {
        cui_outln!(self.cui, "DAPからの削除中...")?;

        match murack_core_data_file::delete_path_str(&self.config.dap_lib, &self.args.path) {
            Ok(_) => Ok(()),
            Err(e) => match e.downcast_ref() {
                //パスが見つからないエラーなら、出力してこの関数はOK
                Some(DomainError::FilePathStrNotFound { .. }) => {
                    self.cui.err(format_args!("{e}\n"))?;
                    Ok(())
                }
                _ => Err(e),
            },
        }
    }

    /// PCから削除
    pub fn remove_pc(&self) -> Result<()> {
        cui_outln!(self.cui, "PCからの削除中...")?;

        match murack_core_data_file::trash_path_str(&self.config.pc_lib, &self.args.path) {
            Ok(_) => Ok(()),
            Err(e) => match e.downcast_ref() {
                //パスが見つからないエラーなら、出力してこの関数はOK
                Some(DomainError::FilePathStrNotFound { .. }) => {
                    self.cui.err(format_args!("{e}\n"))?;
                    Ok(())
                }
                _ => Err(e),
            },
        }
    }
}

/// コマンドの引数
pub struct CommandRemoveArgs {
    /// 削除対象のパス
    ///
    /// ディレクトリ指定可
    pub path: NonEmptyString,
}

impl CommandRemoveArgs {
    /// コマンドの引数を解析
    pub fn parse(command_line: &[String]) -> Result<CommandRemoveArgs> {
        match command_line {
            [s, ..] => Ok(CommandRemoveArgs {
                path: s.clone().try_into()?,
            }),
            [] => Err(Error::InvalidCommandArgument {
                msg: "target path is not specified.".to_owned(),
            }
            .into()),
        }
    }
}
