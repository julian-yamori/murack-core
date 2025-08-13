use murack_core_domain::path::LibraryTrackPath;

use crate::track_data::AudioMetadata;

/// DBに保存されている、AudioMetadata のエンティティ
#[derive(Debug, PartialEq)]
pub struct DbTrackEntity {
    /// 曲のID
    pub id: i32,

    /// 曲ファイルのライブラリ内パス
    pub path: LibraryTrackPath,

    /// 曲の情報本体
    pub metadata: AudioMetadata,
}
