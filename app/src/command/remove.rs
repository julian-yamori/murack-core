use anyhow::Result;
use murack_core_domain::{
    Error as DomainError, db::DbTransaction, path::LibPathStr, song::SongUsecase,
};
use sqlx::PgPool;

use crate::{Config, Error, cui::Cui};

/// removeコマンド
///
/// ライブラリから曲を削除
pub struct CommandRemove<'config, 'cui, CUI, SS>
where
    CUI: Cui,
    SS: SongUsecase,
{
    args: CommandRemoveArgs,
    config: &'config Config,
    cui: &'cui CUI,
    song_usecase: SS,
}

impl<'config, 'cui, CUI, SS> CommandRemove<'config, 'cui, CUI, SS>
where
    CUI: Cui,
    SS: SongUsecase,
{
    pub fn new(
        args: CommandRemoveArgs,
        config: &'config Config,
        cui: &'cui CUI,
        song_usecase: SS,
    ) -> Result<Self> {
        Ok(Self {
            args,
            config,
            cui,
            song_usecase,
        })
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
        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };
        let song_path_list = self
            .song_usecase
            .delete_path_str_db(&mut tx, &self.args.path)
            .await?;
        tx.commit().await?;

        if song_path_list.is_empty() {
            self.cui.err(format_args!(
                "{}\n",
                DomainError::DbPathStrNotFound(self.args.path.clone()),
            ))?;
            return Ok(());
        }

        cui_outln!(self.cui, "以下の曲をDBから削除しました。")?;
        for path in song_path_list {
            cui_outln!(self.cui, "{}", path)?;
        }
        cui_outln!(self.cui)?;

        Ok(())
    }

    /// DAPから削除
    pub fn remove_dap(&self) -> Result<()> {
        cui_outln!(self.cui, "DAPからの削除中...")?;

        match self
            .song_usecase
            .delete_path_str_dap(&self.config.dap_lib, &self.args.path)
        {
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

        match self
            .song_usecase
            .delete_path_str_pc(&self.config.pc_lib, &self.args.path)
        {
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
    pub path: LibPathStr,
}

impl CommandRemoveArgs {
    /// コマンドの引数を解析
    pub fn parse(command_line: &[String]) -> Result<CommandRemoveArgs> {
        match command_line {
            [s, ..] => Ok(CommandRemoveArgs {
                path: s.clone().into(),
            }),
            [] => Err(Error::InvalidCommandArgument {
                msg: "target path is not specified.".to_owned(),
            }
            .into()),
        }
    }
}
