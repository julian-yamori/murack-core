use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;

/// ソートの種類
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "playlist_type", rename_all = "lowercase")]
pub enum SortType {
    /// 曲名
    TrackName,
    /// アーティスト
    Artist,
    /// アルバム
    Album,
    /// ジャンル
    Genre,
    /// プレイリスト並び順
    Playlist,
    /// 作曲者
    Composer,
    /// 再生時間
    Duration,
    /// トラック番号
    TrackIndex,
    /// ディスク番号
    DiscIndex,
    /// リリース日
    ReleaseDate,
    /// レート
    Rating,
    /// 登録日
    EntryDate,
    /// パス
    Path,
}
