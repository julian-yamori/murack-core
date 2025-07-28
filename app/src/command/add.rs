use anyhow::Result;
use murack_core_domain::{
    Error as DomainError,
    path::{LibPathStr, LibTrackPath},
    sync::SyncUsecase,
};
use sqlx::PgPool;

use crate::{Config, Error, cui::Cui};

/// addコマンド
///
/// 曲をライブラリに追加する
pub struct CommandAdd<'config, 'cui, CUI, SS>
where
    CUI: Cui,
    SS: SyncUsecase,
{
    args: CommandAddArgs,

    config: &'config Config,
    cui: &'cui CUI,
    sync_usecase: SS,
}

impl<'config, 'cui, CUI, SS> CommandAdd<'config, 'cui, CUI, SS>
where
    CUI: Cui,
    SS: SyncUsecase,
{
    pub fn new(
        args: CommandAddArgs,
        config: &'config Config,
        cui: &'cui CUI,
        sync_usecase: SS,
    ) -> Self {
        Self {
            args,
            config,
            cui,
            sync_usecase,
        }
    }

    /// このコマンドを実行
    pub async fn run(&self, db_pool: &PgPool) -> Result<()> {
        //指定されたパスから音声ファイルを検索
        let path_list =
            murack_core_data_file::search_by_lib_path(&self.config.pc_lib, &self.args.path)?;

        let file_count = path_list.len();
        if file_count == 0 {
            return Err(DomainError::FilePathStrNotFound {
                lib_root: self.config.pc_lib.clone(),
                path_str: self.args.path.clone(),
            }
            .into());
        }

        //取得した全ファイルについて処理
        for (track_idx, track_lib_path) in path_list.iter().enumerate() {
            self.write_console_progress(track_idx, file_count, track_lib_path)?;

            if let Err(e) = self.unit_add(db_pool, track_lib_path).await {
                self.cui.err(format_args!("{e}\n"))?;
            }
        }

        Ok(())
    }

    /// 曲1個単位の追加処理
    ///
    /// # Arguments
    /// - track_path: 作業対象の曲のパス
    /// - entry_date: 登録日
    async fn unit_add(&self, db_pool: &PgPool, track_path: &LibTrackPath) -> Result<()> {
        //PCファイル情報読み込み
        let mut pc_track = murack_core_data_file::read_track_sync(&self.config.pc_lib, track_path)?;

        //DBに登録
        let mut tx = db_pool.begin().await?;
        self.sync_usecase
            .register_db(&mut tx, track_path, &mut pc_track)
            .await?;
        tx.commit().await?;

        //PCからDAPにコピー
        murack_core_data_file::copy_track_over_lib(
            &self.config.pc_lib,
            &self.config.dap_lib,
            track_path,
        )?;

        Ok(())
    }

    /// コンソールに進捗を出力
    ///
    /// # Arguments
    /// - current_idx: 何番目の曲の処理中か(0始点)
    /// - all_count: 全部で何曲あるか
    /// - track_path: 作業中の曲のパス
    fn write_console_progress(
        &self,
        current_idx: usize,
        all_count: usize,
        track_path: &LibTrackPath,
    ) -> anyhow::Result<()> {
        cui_outln!(
            self.cui,
            "({}/{}) {}",
            current_idx + 1,
            all_count,
            track_path
        )
    }
}

/// add コマンドの引数
pub struct CommandAddArgs {
    /// 追加対象のパス
    pub path: LibPathStr,
}

impl CommandAddArgs {
    /// コマンドの引数を解析
    pub fn parse(command_line: &[String]) -> Result<CommandAddArgs> {
        match command_line {
            [s, ..] => Ok(CommandAddArgs {
                path: s.clone().try_into()?,
            }),
            [] => Err(Error::InvalidCommandArgument {
                msg: "target path is not specified.".to_owned(),
            }
            .into()),
        }
    }
}
