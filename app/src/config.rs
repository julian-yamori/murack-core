use crate::Error;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};
use toml::{value::Table, Value};

/// 設定ファイル取扱
#[derive(Debug, PartialEq)]
pub struct Config {
    /// PC側のライブラリのルートパス
    pub pc_lib: PathBuf,
    /// DAP側のライブラリのルートパス
    pub dap_lib: PathBuf,
    /// DAPにプレイリストファイルを配置するパス
    pub dap_playlist: PathBuf,
    /// WalkStudioのDBファイルパス
    pub db_path: PathBuf,
}

impl Config {
    /// 設定ファイルを読み込む
    ///
    /// # Arguments
    /// - path: 設定tomlファイルのパス
    pub fn load(path: &Path) -> Result<Self> {
        let file_str = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => return Err(domain::Error::FileIoError(path.to_owned(), e).into()),
        };
        let root_v = file_str
            .parse::<Value>()
            .with_context(|| "Failed to parse config.")?;
        let root = match root_v.as_table() {
            Some(t) => t,
            None => return Err(Error::ConfigRootIsNotTable.into()),
        };

        Ok(Self {
            pc_lib: tb_path(root, "pc_lib")?,
            dap_lib: tb_path(root, "dap_lib")?,
            dap_playlist: tb_path(root, "dap_playlist")?,
            db_path: tb_path(root, "db_path")?,
        })
    }

    /// テスト用のダミー値を返す
    #[cfg(test)]
    pub fn dummy() -> Self {
        Self {
            pc_lib: "pc_lib".into(),
            dap_lib: "dap_lib".into(),
            dap_playlist: "dap_playlist".into(),
            db_path: "db_path".into(),
        }
    }
}

/// TOMLのテーブルから文字列値を取得
fn tb_str<'a>(t: &'a Table, key: &str) -> Result<&'a str> {
    match t.get(key) {
        Some(v) => match v.as_str() {
            Some(s) => Ok(s),
            None => Err(Error::ConfigNotString {
                key: key.to_owned(),
            }
            .into()),
        },
        None => Err(Error::ConfigNotFound {
            key: key.to_owned(),
        }
        .into()),
    }
}
/// TOMLのテーブルからパス値を取得
fn tb_path(t: &Table, key: &str) -> Result<PathBuf> {
    Ok(tb_str(t, key)?.into())
}
