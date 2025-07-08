use crate::{db_wrapper::TransactionWrapper, path::LibSongPath, playlist::Playlist};
use anyhow::Result;
use mockall::automock;

/// 曲データの検索機能
/// #todo
/// 現状ではWalkBaseでの用途が限定的だし、
/// やたらとごついし、
/// 整理したいやつ。
///
/// とりあえずdapモジュールに定義
#[automock]
pub trait SongFinder {
    /// プレイリストに含まれる曲のパスリストを取得
    /// # Arguments
    /// - plist 取得対象のプレイリスト情報
    fn get_song_path_list<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        plist: &Playlist,
    ) -> Result<Vec<LibSongPath>>;
}
