use num_derive::FromPrimitive;

/// ソートの種類
#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive)]
pub enum SortType {
    /// 曲名
    SongName = 0,
    /// アーティスト
    Artist = 1,
    /// アルバム
    Album = 2,
    /// ジャンル
    Genre = 3,
    /// プレイリスト並び順
    Playlist = 4,
    /// 作曲者
    Composer = 5,
    /// 再生時間
    Duration = 6,
    /// トラック番号
    TrackIndex = 7,
    /// ディスク番号
    DiscIndex = 8,
    /// リリース日
    ReleaseDate = 9,
    /// レート
    Rating = 10,
    /// 登録日
    EntryDate = 11,
    /// パス
    Path = 12,
}
