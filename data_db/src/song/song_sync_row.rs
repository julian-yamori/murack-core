use crate::converts::{DbDate, DbOptionString};

/// SongSyncの、songテーブルレコードの部分
pub struct SongSyncRow {
    /// 曲ID
    pub rowid: i32,

    /// 曲の長さ(ミリ秒)
    pub duration: u32,

    /// 曲名
    pub title: DbOptionString,

    /// アーティスト
    pub artist: DbOptionString,
    /// アルバム
    pub album: DbOptionString,
    /// ジャンル
    pub genre: DbOptionString,
    /// アルバムアーティスト
    pub album_artist: DbOptionString,
    /// 作曲者
    pub composer: DbOptionString,

    /// トラック番号
    pub track_number: Option<i32>,
    /// トラック最大数
    pub track_max: Option<i32>,

    /// ディスク番号
    pub disc_number: Option<i32>,
    /// ディスク番号(最大)
    pub disc_max: Option<i32>,

    /// リリース日
    pub release_date: Option<DbDate>,

    /// メモ
    pub memo: DbOptionString,

    /// 歌詞
    pub lyrics: DbOptionString,
}
