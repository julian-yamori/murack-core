use super::SongSync;
use crate::path::LibSongPath;

/// DBに保存されている、PC・DB間で同期するべき曲の情報
#[derive(Debug, PartialEq)]
pub struct DbSongSync {
    /// 曲のID
    pub id: i32,

    /// 曲ファイルのライブラリ内パス
    pub path: LibSongPath,

    /// 曲の情報本体
    pub song_sync: SongSync,
}
