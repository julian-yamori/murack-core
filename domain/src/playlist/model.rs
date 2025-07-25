use crate::filter::RootFilter;

use super::{PlaylistType, SortType};

/// プレイリスト情報
/// # todo
/// とりあえず雑にPlaylist行全体とかツリーとか保持
#[derive(Debug, PartialEq)]
pub struct Playlist {
    /// プレイリストID
    pub rowid: i32,

    /// プレイリストの種類
    pub playlist_type: PlaylistType,

    /// プレイリスト名
    pub name: String,

    /// 親プレイリストID
    ///
    /// 親になれるプレイリストはEPlaylistType.Folderのみ。
    /// 最上位ならNone。
    ///
    /// # todo
    /// この辺もModelの制約として整理したほうがいい
    pub parent_id: Option<i32>,

    /// 親プレイリスト内でのインデックス
    pub in_folder_order: u32,

    /// PlaylistType::Filter で使うフィルタ
    pub filter: Option<RootFilter>,

    /// ソート対象
    pub sort_type: SortType,

    /// ソートが降順か
    pub sort_desc: bool,

    /// DAPにこのプレイリストを保存するか
    pub save_dap: bool,

    /// リスト内容がPlaylistTrackテーブルにリストアップ済みか
    ///
    /// 更新されうる処理が行われるごとに、
    /// FilterとFolderのフラグが解除される。
    ///
    /// Normalでは常にtrue。
    pub listuped_flag: bool,

    /// 前回DAPに反映してから、リストが変更されたか
    pub dap_changed: bool,

    /// 子プレイリストの一覧
    ///
    /// ツリー構築時のみ有効
    pub children: Vec<Playlist>,

    /// 親プレイリストの名前リスト
    ///
    /// ツリー構築時のみ有効
    pub parent_names: Vec<String>,
}
