use domain::playlist::{Playlist, PlaylistType, SortType};

/// playlistテーブルのレコード
pub struct PlaylistRow {
    /// プレイリストID
    pub id: i32,

    /// プレイリストの種類
    pub playlist_type: PlaylistType,

    /// プレイリスト名
    pub name: String,

    /// 親プレイリストID
    ///
    /// 親になれるプレイリストはPlaylistType.Folderのみ。
    /// 最上位ならNone。
    pub parent_id: Option<i32>,

    /// 親プレイリスト内でのインデックス
    pub in_folder_order: i32,

    /// PlaylistType::Filter で使うフィルタ
    pub filter_json: Option<serde_json::Value>,

    /// ソート対象
    pub sort_type: SortType,

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

impl TryFrom<PlaylistRow> for Playlist {
    type Error = anyhow::Error;

    fn try_from(row: PlaylistRow) -> anyhow::Result<Self> {
        Ok(Self {
            rowid: row.id,
            playlist_type: row.playlist_type,
            name: row.name,
            parent_id: row.parent_id,
            in_folder_order: row.in_folder_order as u32,
            filter: match row.filter_json {
                Some(json) => Some(serde_json::from_value(json)?),
                None => None,
            },
            sort_type: row.sort_type,
            sort_desc: row.sort_desc,
            save_dap: row.save_dap,
            listuped_flag: row.listuped_flag,
            dap_changed: row.dap_changed,
            //todo 以下は暫定
            children: vec![],
            parent_names: vec![],
        })
    }
}
