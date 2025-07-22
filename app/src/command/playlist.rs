use anyhow::Result;
use murack_core_domain::dap::{DapPlaylistObserver, DapPlaylistUsecase};
use sqlx::PgPool;

use crate::{Config, cui::Cui};

/// playlistコマンド
///
/// DAPのプレイリストを更新する
pub struct CommandPlaylist<'config, 'cui, CUI, DPS>
where
    CUI: Cui + Send + Sync,
    DPS: DapPlaylistUsecase,
{
    config: &'config Config,
    cui: &'cui CUI,
    dap_playlist_usecase: DPS,
}

impl<'config, 'cui, CUI, DPS> CommandPlaylist<'config, 'cui, CUI, DPS>
where
    CUI: Cui + Send + Sync,
    DPS: DapPlaylistUsecase,
{
    pub fn new(config: &'config Config, cui: &'cui CUI, dap_playlist_usecase: DPS) -> Self {
        Self {
            config,
            cui,
            dap_playlist_usecase,
        }
    }

    /// このコマンドを実行
    pub async fn run(self, db_pool: &PgPool) -> Result<()> {
        let mut observer = Observer { cui: self.cui };

        self.dap_playlist_usecase
            .run(db_pool, &self.config.dap_playlist, false, &mut observer)
            .await
    }
}

struct Observer<'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    cui: &'cui CUI,
}

impl<'cui, CUI> DapPlaylistObserver for Observer<'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    /// プレイリスト情報の読み込み開始時
    fn on_start_load_playlist(&mut self) {
        cui_outln!(self.cui, "プレイリスト情報の取得中...").unwrap()
    }
    /// ファイルの保存開始時
    fn on_start_save_file(&mut self) {
        cui_outln!(self.cui, "プレイリストファイルの保存中...").unwrap()
    }
}
