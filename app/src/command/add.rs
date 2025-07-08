use crate::{cui::Cui, AppComponents, Config, Error};
use anyhow::Result;
use chrono::{Local, NaiveDate};
use domain::{
    db_wrapper::{ConnectionFactory, ConnectionWrapper},
    path::{LibPathStr, LibSongPath},
    sync::SyncUsecase,
    FileLibraryRepository,
};
use std::rc::Rc;

/// addコマンド
///
/// 曲をライブラリに追加する
pub struct CommandAdd {
    args: Args,

    config: Rc<Config>,
    cui: Rc<dyn Cui>,
    connection_factory: Rc<ConnectionFactory>,
    file_library_repository: Rc<dyn FileLibraryRepository>,
    sync_usecase: Rc<dyn SyncUsecase>,
}

impl CommandAdd {
    pub fn new(command_line: &[String], app_components: &impl AppComponents) -> Result<Self> {
        Ok(Self {
            args: parse_args(command_line)?,
            config: app_components.config().clone(),
            cui: app_components.cui().clone(),
            connection_factory: app_components.connection_factory().clone(),
            file_library_repository: app_components.file_library_repository().clone(),
            sync_usecase: app_components.sync_usecase().clone(),
        })
    }

    /// このコマンドを実行
    pub fn run(&self) -> Result<()> {
        let entry_date = Local::today().naive_local();

        //指定されたパスから音声ファイルを検索
        let path_list = self
            .file_library_repository
            .search_by_lib_path(&self.config.pc_lib, &self.args.path)?;

        let file_count = path_list.len();
        if file_count == 0 {
            return Err(domain::Error::FilePathStrNotFound {
                lib_root: self.config.pc_lib.clone(),
                path_str: self.args.path.clone(),
            }
            .into());
        }

        let mut db = self.connection_factory.open()?;

        //取得した全ファイルについて処理
        for (song_idx, song_lib_path) in path_list.iter().enumerate() {
            self.write_console_progress(song_idx, file_count, song_lib_path);

            if let Err(e) = self.unit_add(&mut db, song_lib_path, entry_date) {
                self.cui.err(format_args!("{}\n", e));
            }
        }

        Ok(())
    }

    /// 曲1個単位の追加処理
    ///
    /// # Arguments
    /// - song_path: 作業対象の曲のパス
    /// - entry_date: 登録日
    fn unit_add(
        &self,
        db: &mut ConnectionWrapper,
        song_path: &LibSongPath,
        entry_date: NaiveDate,
    ) -> Result<()> {
        //PCファイル情報読み込み
        let mut pc_song = self
            .file_library_repository
            .read_song_sync(&self.config.pc_lib, song_path)?;

        //DBに登録
        db.run_in_transaction(|tx| {
            self.sync_usecase
                .register_db(tx, song_path, &mut pc_song, entry_date)
        })?;

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
    ) {
        cui_outln!(
            self.cui,
            "({}/{}) {}",
            current_idx + 1,
            all_count,
            song_path
        );
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
