use murack_core_domain::path::LibraryTrackPath;

use crate::audio_metadata::AudioMetadata;

/// DBに保存されている、PC・DB間で同期するべき曲の情報
#[derive(Debug, PartialEq)]
pub struct DbTrackSync {
    /// 曲のID
    pub id: i32,

    /// 曲ファイルのライブラリ内パス
    pub path: LibraryTrackPath,

    /// 曲の情報本体
    pub metadata: AudioMetadata,
}
