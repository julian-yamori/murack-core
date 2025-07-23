use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use murack_core_domain::Error as DomainError;
use serde::{Deserialize, Serialize};

/// 設定ファイル取扱
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// PC側のライブラリのルートパス
    pub pc_lib: PathBuf,
    /// DAP側のライブラリのルートパス
    pub dap_lib: PathBuf,
    /// DAPにプレイリストファイルを配置するパス
    pub dap_playlist: PathBuf,
    /// MurackのDBのURL
    pub database_url: String,
}

impl Config {
    /// 設定ファイルを読み込む
    ///
    /// # Arguments
    /// - path: 設定tomlファイルのパス
    pub fn load(path: &Path) -> Result<Self> {
        let file_str = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => return Err(DomainError::FileIoError(path.to_owned(), e).into()),
        };

        let config = toml::from_str(&file_str)?;
        Ok(config)
    }

    /// テスト用のダミー値を返す
    #[cfg(test)]
    pub fn dummy() -> Self {
        Self {
            pc_lib: "pc_lib".into(),
            dap_lib: "dap_lib".into(),
            dap_playlist: "dap_playlist".into(),
            database_url: "database_url".to_string(),
        }
    }
}
