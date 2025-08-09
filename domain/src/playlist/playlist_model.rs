use sqlx::PgTransaction;

use crate::{
    NonEmptyString, SortType,
    filter::RootFilter,
    playlist::{PlaylistRow, PlaylistType, playlist_error::PlaylistError},
};

/// プレイリスト情報
/// # todo
/// とりあえず雑にPlaylist行全体とかツリーとか保持
#[derive(Debug, PartialEq)]
pub struct Playlist {
    /// プレイリストID
    pub id: i32,

    /// プレイリストの種類
    pub playlist_type: PlaylistType,

    /// プレイリスト名
    pub name: NonEmptyString,

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
}

/// IDを指定してプレイリストを検索
pub async fn get_playlist<'c>(
    tx: &mut PgTransaction<'c>,
    playlist_id: i32,
) -> Result<Option<Playlist>, PlaylistError> {
    let opt = sqlx::query_as!(
            PlaylistRow,
            r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name AS "name: NonEmptyString", parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists WHERE id = $1"#,
            playlist_id
        )
        .fetch_optional(&mut **tx)
        .await?;

    match opt {
        Some(row) => Ok(Some(row.try_into()?)),
        None => Ok(None),
    }
}
