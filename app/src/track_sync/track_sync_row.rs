use chrono::NaiveDate;

/// TrackSyncの、trackテーブルレコードの部分
pub struct TrackSyncRow {
    /// 曲ID
    pub id: i32,

    /// 曲の長さ(ミリ秒)
    pub duration: i32,

    /// 曲名
    pub title: String,

    /// アーティスト
    pub artist: String,
    /// アルバム
    pub album: String,
    /// ジャンル
    pub genre: String,
    /// アルバムアーティスト
    pub album_artist: String,
    /// 作曲者
    pub composer: String,

    /// トラック番号
    pub track_number: Option<i32>,
    /// トラック最大数
    pub track_max: Option<i32>,

    /// ディスク番号
    pub disc_number: Option<i32>,
    /// ディスク番号(最大)
    pub disc_max: Option<i32>,

    /// リリース日
    pub release_date: Option<NaiveDate>,

    /// メモ
    pub memo: String,

    /// 歌詞
    pub lyrics: String,
}
