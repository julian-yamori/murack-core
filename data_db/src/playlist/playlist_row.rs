use crate::converts::{DbPlaylistType, DbSortType};
use domain::playlist::Playlist;

/// playlistテーブルのレコード
pub struct PlaylistRow {
    /// プレイリストID
    pub rowid: i32,

    /// プレイリストの種類
    pub playlist_type: DbPlaylistType,

    /// プレイリスト名
    pub name: String,

    /// 親プレイリストID
    ///
    /// 親になれるプレイリストはPlaylistType.Folderのみ。
    /// 最上位ならNone。
    pub parent_id: Option<i32>,

    /// 親プレイリスト内でのインデックス
    pub in_folder_order: u32,

    /// このプレイリスト用フィルタの基底ノードの、FilterData.Id
    ///
    /// PlaylistType.Filter用。
    /// フィルタがなければNone。
    pub filter_root_id: Option<i32>,

    /// ソート対象
    pub sort_type: DbSortType,

    /// ソートが降順か
    pub sort_desc: bool,

    /// DAPにこのプレイリストを保存するか
    pub save_dap: bool,

    /// リスト内容がPlaylistSongテーブルにリストアップ済みか
    ///
    /// 更新されうる処理が行われるごとに、
    /// FilterとFolderのフラグが解除される。
    ///
    /// Normalでは常にtrue
    pub listuped_flag: bool,

    /// 前回DAPに反映してから、リストが変更されたか
    pub dap_changed: bool,
}

impl From<PlaylistRow> for Playlist {
    fn from(row: PlaylistRow) -> Self {
        Self {
            rowid: row.rowid,
            playlist_type: row.playlist_type.into(),
            name: row.name,
            parent_id: row.parent_id,
            in_folder_order: row.in_folder_order,
            filter_root_id: row.filter_root_id,
            sort_type: row.sort_type.into(),
            sort_desc: row.sort_desc,
            save_dap: row.save_dap,
            listuped_flag: row.listuped_flag,
            dap_changed: row.dap_changed,
            //todo 以下は暫定
            children: vec![],
            parent_names: vec![],
        }
    }
}
