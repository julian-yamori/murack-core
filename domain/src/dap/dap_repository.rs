use anyhow::Result;
use mockall::automock;
use std::path::Path;

/// DAP内のファイルを扱うリポジトリ
#[automock]
pub trait DapRepository {
    /// DAPにあるプレイリストファイルを列挙
    /// # Arguments
    /// - dap_plist_path: DAPのプレイリスト保存パス
    fn listup_playlist_files(&self, dap_plist_path: &Path) -> Result<Vec<String>>;

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
    ) -> Result<()>;

    /// DAPのプレイリストファイルを削除
    /// # Arguments
    /// - dap_plist_path: DAPのプレイリスト保存パス
    /// - name: プレイリストファイル名
    fn delete_playlist_file(&self, dap_plist_path: &Path, file_name: &str) -> Result<()>;
}
