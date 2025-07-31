use anyhow::Result;
use murack_core_domain::{NonEmptyString, path::LibraryTrackPath};
use sqlx::{PgPool, PgTransaction};

use crate::{
    Config, DbTrackError,
    cui::Cui,
    data_file::{self, LibraryFsError},
    db_common,
};

/// removeコマンド
///
/// ライブラリから曲を削除
pub struct CommandRemove<'config, 'cui, CUI>
where
    CUI: Cui,
{
    args: CommandRemoveArgs,
    config: &'config Config,
    cui: &'cui CUI,
}

impl<'config, 'cui, CUI> CommandRemove<'config, 'cui, CUI>
where
    CUI: Cui,
{
    pub fn new(args: CommandRemoveArgs, config: &'config Config, cui: &'cui CUI) -> Self {
        Self { args, config, cui }
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
        let track_path_list = delete_path_str_db(&mut tx, &self.args.path).await?;
        tx.commit().await?;

        if track_path_list.is_empty() {
            self.cui.err(format_args!(
                "{}\n",
                DbTrackError::DbPathStrNotFound(self.args.path.clone()),
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

        match data_file::delete_path_str(&self.config.dap_lib, &self.args.path) {
            Ok(_) => Ok(()),
            Err(e) => match e.downcast_ref() {
                //パスが見つからないエラーなら、出力してこの関数はOK
                Some(LibraryFsError::FilePathStrNotFound { .. }) => {
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

        match data_file::trash_path_str(&self.config.pc_lib, &self.args.path) {
            Ok(_) => Ok(()),
            Err(e) => match e.downcast_ref() {
                //パスが見つからないエラーなら、出力してこの関数はOK
                Some(LibraryFsError::FilePathStrNotFound { .. }) => {
                    self.cui.err(format_args!("{e}\n"))?;
                    Ok(())
                }
                _ => Err(e),
            },
        }
    }
}

/// パス文字列を指定してDBから削除
///
/// # Arguments
/// - path: 削除する曲のパス
///
/// # Returns
/// 削除した曲のパスリスト
async fn delete_path_str_db<'c>(
    tx: &mut PgTransaction<'c>,
    path_str: &NonEmptyString,
) -> Result<Vec<LibraryTrackPath>> {
    let track_path_list = db_common::track_paths_by_path_str(tx, path_str).await?;

    for path in &track_path_list {
        db_common::delete_track_db(tx, path).await?;
    }

    Ok(track_path_list)
}

/// コマンドの引数
pub struct CommandRemoveArgs {
    /// 削除対象のパス
    ///
    /// ディレクトリ指定可
    pub path: NonEmptyString,
}
