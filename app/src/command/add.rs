use anyhow::Result;
use murack_core_domain::{NonEmptyString, path::LibraryTrackPath};
use sqlx::PgPool;

use crate::{
    Config,
    audio_metadata::file_io,
    cui::Cui,
    data_file::{self, LibraryFsError},
    db_common,
};

/// addコマンド
///
/// 曲をライブラリに追加する
pub struct CommandAdd<'config, 'cui, CUI>
where
    CUI: Cui,
{
    args: CommandAddArgs,

    config: &'config Config,
    cui: &'cui CUI,
}

impl<'config, 'cui, CUI> CommandAdd<'config, 'cui, CUI>
where
    CUI: Cui,
{
    pub fn new(args: CommandAddArgs, config: &'config Config, cui: &'cui CUI) -> Self {
        Self { args, config, cui }
    }

    /// このコマンドを実行
    pub async fn run(&self, db_pool: &PgPool) -> Result<()> {
        //指定されたパスから音声ファイルを検索
        let path_list = data_file::search_by_lib_path(&self.config.pc_lib, &self.args.path)?;

        let file_count = path_list.len();
        if file_count == 0 {
            return Err(LibraryFsError::FilePathStrNotFound {
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
    async fn unit_add(&self, db_pool: &PgPool, track_path: &LibraryTrackPath) -> Result<()> {
        //PCファイル情報読み込み
        let pc_track = file_io::read_track_sync(&self.config.pc_lib, track_path)?;

        //DBに登録
        db_common::add_track_to_db(db_pool, track_path, pc_track).await?;

        //PCからDAPにコピー
        data_file::copy_track_over_lib(&self.config.pc_lib, &self.config.dap_lib, track_path)?;

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
        track_path: &LibraryTrackPath,
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
    pub path: NonEmptyString,
}
