use super::Playlist;
use crate::db_wrapper::TransactionWrapper;
use anyhow::Result;
use mockall::automock;

/// プレイリスト関係のDBリポジトリ
#[automock]
pub trait DbPlaylistRepository {
    /// IDを指定してプレイリストを検索
    /// # Arguments
    /// id: playlist.rowid
    fn get_playlist<'c>(&self, tx: &TransactionWrapper<'c>, id: i32) -> Result<Option<Playlist>>;

    /// プレイリストのツリー構造を取得
    /// # Returns
    /// 最上位プレイリストのリスト
    fn get_playlist_tree<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<Vec<Playlist>>;

    /// 全フィルタプレイリスト・フォルダプレイリストの、リストアップ済みフラグを解除する。
    fn reset_listuped_flag<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<()>;

    /// 全プレイリストの、Walkmanに保存してからの変更フラグを設定
    /// # Arguments
    /// - is_changed: 変更されたか
    fn set_dap_change_flag_all<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        is_changed: bool,
    ) -> Result<()>;
}
