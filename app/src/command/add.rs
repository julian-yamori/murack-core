use anyhow::Result;
use murack_core_domain::{
    Error as DomainError, FileLibraryRepository,
    db::DbTransaction,
    path::{LibPathStr, LibSongPath},
    sync::SyncUsecase,
};
use sqlx::PgPool;

use crate::{Config, Error, cui::Cui};

/// addコマンド
///
/// 曲をライブラリに追加する
pub struct CommandAdd<'config, 'cui, CUI, FR, SS>
where
    CUI: Cui,
    FR: FileLibraryRepository,
    SS: SyncUsecase,
{
    args: Args,

    config: &'config Config,
    cui: &'cui CUI,
    file_library_repository: FR,
    sync_usecase: SS,
}

impl<'config, 'cui, CUI, FR, SS> CommandAdd<'config, 'cui, CUI, FR, SS>
where
    CUI: Cui,
    FR: FileLibraryRepository,
    SS: SyncUsecase,
{
    pub fn new(
        command_line: &[String],
        config: &'config Config,
        cui: &'cui CUI,
        file_library_repository: FR,
        sync_usecase: SS,
    ) -> Result<Self> {
        Ok(Self {
            args: parse_args(command_line)?,
            config,
            cui,
            file_library_repository,
            sync_usecase,
        })
    }

    /// このコマンドを実行
    pub async fn run(&self, db_pool: &PgPool) -> Result<()> {
        //指定されたパスから音声ファイルを検索
        let path_list = self
            .file_library_repository
            .search_by_lib_path(&self.config.pc_lib, &self.args.path)?;

        let file_count = path_list.len();
        if file_count == 0 {
            return Err(DomainError::FilePathStrNotFound {
                lib_root: self.config.pc_lib.clone(),
                path_str: self.args.path.clone(),
            }
            .into());
        }

        //取得した全ファイルについて処理
        for (song_idx, song_lib_path) in path_list.iter().enumerate() {
            self.write_console_progress(song_idx, file_count, song_lib_path)?;

            if let Err(e) = self.unit_add(db_pool, song_lib_path).await {
                self.cui.err(format_args!("{e}\n"))?;
            }
        }

        Ok(())
    }

    /// 曲1個単位の追加処理
    ///
    /// # Arguments
    /// - song_path: 作業対象の曲のパス
    /// - entry_date: 登録日
    async fn unit_add(&self, db_pool: &PgPool, song_path: &LibSongPath) -> Result<()> {
        //PCファイル情報読み込み
        let mut pc_song = self
            .file_library_repository
            .read_song_sync(&self.config.pc_lib, song_path)?;

        //DBに登録
        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };
        self.sync_usecase
            .register_db(&mut tx, song_path, &mut pc_song)
            .await?;
        tx.commit().await?;

        //PCからDAPにコピー
        self.file_library_repository.copy_song_over_lib(
            &self.config.pc_lib,
            &self.config.dap_lib,
            song_path,
        )?;

        Ok(())
    }

    /// コンソールに進捗を出力
    ///
    /// # Arguments
    /// - current_idx: 何番目の曲の処理中か(0始点)
    /// - all_count: 全部で何曲あるか
    /// - song_path: 作業中の曲のパス
    fn write_console_progress(
        &self,
        current_idx: usize,
        all_count: usize,
        song_path: &LibSongPath,
    ) -> anyhow::Result<()> {
        cui_outln!(
            self.cui,
            "({}/{}) {}",
            current_idx + 1,
            all_count,
            song_path
        )
    }
}

/// コマンドの引数
struct Args {
    /// 追加対象のパス
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
