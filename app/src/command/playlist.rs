use crate::{AppComponents, Config, cui::Cui};
use anyhow::Result;
use domain::{
    dap::{DapPlaylistObserver, DapPlaylistUsecase},
    db_wrapper::ConnectionFactory,
};
use std::rc::Rc;

/// playlistコマンド
///
/// DAPのプレイリストを更新する
pub struct CommandPlaylist {
    config: Rc<Config>,
    cui: Rc<dyn Cui>,
    connection_factory: Rc<ConnectionFactory>,

    dap_playlist_usecase: Rc<dyn DapPlaylistUsecase>,
}

impl CommandPlaylist {
    pub fn new(app_components: &impl AppComponents) -> Self {
        Self {
            config: app_components.config().clone(),
            cui: app_components.cui().clone(),
            connection_factory: app_components.connection_factory().clone(),
            dap_playlist_usecase: app_components.dap_playlist_usecase().clone(),
        }
    }

    /// このコマンドを実行
    pub fn run(&self) -> Result<()> {
        let mut observer = Observer {
            cui: self.cui.clone(),
        };
        let mut db = self.connection_factory.open()?;

        self.dap_playlist_usecase
            .run(&mut db, &self.config.dap_playlist, false, &mut observer)
    }
}

struct Observer {
    cui: Rc<dyn Cui>,
}

impl DapPlaylistObserver for Observer {
    /// プレイリスト情報の読み込み開始時
    fn on_start_load_playlist(&mut self) {
        cui_outln!(self.cui, "プレイリスト情報の取得中...")
    }
    /// ファイルの保存開始時
    fn on_start_save_file(&mut self) {
        cui_outln!(self.cui, "プレイリストファイルの保存中...")
    }
}
