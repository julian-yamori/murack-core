use anyhow::{Context, Result};
use domain::dap::DapRepository;
use std::{
    fs::{self, Metadata},
    path::Path,
};

/// DapRepositoryの本実装
pub struct DapRepositoryImpl {}

impl DapRepository for DapRepositoryImpl {
    /// DAPにあるプレイリストファイルを列挙
    /// # Arguments
    /// - dap_plist_path: DAPのプレイリスト保存パス
    fn listup_playlist_files(&self, dap_plist_path: &Path) -> Result<Vec<String>> {
        let entries = fs::read_dir(dap_plist_path)
            .map_err(|e| domain::Error::FileIoError(dap_plist_path.to_owned(), e))?;

        let mut ret_vec = Vec::new();

        //ディレクトリ内のファイルを列挙
        for entry in entries {
            let entry = entry.with_context(|| {
                format!("failed to get file entry in: {}", dap_plist_path.display())
            })?;

            let entry_path = entry.path();
            let metadata = entry
                .metadata()
                .map_err(|e| domain::Error::FileIoError(entry_path.clone(), e))?;

            //プレイリストならリストに追加
            if is_entry_playlist(&entry_path, &metadata) {
                let name_utf8 = entry_path
                    .file_name()
                    .with_context(|| {
                        format!("ファイル名の取得に失敗しました: {}", entry_path.display())
                    })?
                    .to_str()
                    .with_context(|| {
                        format!(
                            "ファイル名のUTF-8への変換に失敗しました: {}",
                            entry_path.display()
                        )
                    })?
                    .to_owned();
                ret_vec.push(name_utf8);
            }
        }

        Ok(ret_vec)
    }

    /// DAPにプレイリストファイルを作成
    /// # Arguments
    /// - dap_plist_path: DAPのプレイリスト保存パス
    /// - name: プレイリストファイル名
    /// - content: プレイリストファイルの内容
    fn make_playlist_file(
        &self,
        dap_plist_path: &Path,
        file_name: &str,
        content: &str,
    ) -> Result<()> {
        let path = dap_plist_path.join(file_name);

        fs::write(&path, content).map_err(|e| domain::Error::FileIoError(path, e).into())
    }

    /// DAPのプレイリストファイルを削除
    /// # Arguments
    /// - dap_plist_path: DAPのプレイリスト保存パス
    /// - name: プレイリストファイル名
    fn delete_playlist_file(&self, dap_plist_path: &Path, file_name: &str) -> Result<()> {
        let path = dap_plist_path.join(file_name);

        fs::remove_file(&path).map_err(|e| domain::Error::FileIoError(path, e).into())
    }
}

/// プレイリストファイルの拡張子(ピリオドなし)
/// # todo
/// 外部から設定できるようにする
/// Usecaseとも定義が重複
const PLAYLIST_EXT: &str = "m3u";

/// ディレクトリエントリがプレイリストか判別
fn is_entry_playlist(path: &Path, metadata: &Metadata) -> bool {
    if !metadata.is_file() {
        return false;
    }

    let ext_os = match path.extension() {
        Some(e) => e,
        None => return false,
    };
    let ext = match ext_os.to_str() {
        Some(s) => s,
        None => return false,
    };

    ext == PLAYLIST_EXT
}
