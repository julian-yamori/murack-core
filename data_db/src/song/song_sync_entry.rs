use chrono::NaiveDate;

///SongSyncRowの登録用データ
pub struct SongSyncEntry<'a> {
    /// 曲の長さ(ミリ秒)
    pub duration: i32,

    /// 曲名
    pub title: &'a str,

    /// アーティスト
    pub artist: &'a str,
    /// アルバム
    pub album: &'a str,
    /// ジャンル
    pub genre: &'a str,
    /// アルバムアーティスト
    pub album_artist: &'a str,
    /// 作曲者
    pub composer: &'a str,

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
    pub memo: &'a str,

    /// 歌詞
    pub lyrics: &'a str,

    /// 曲名(並び替え用)
    pub title_order: &'a str,
    /// アーティスト(並び替え用)
    pub artist_order: &'a str,
    /// アルバム(並び替え用)
    pub album_order: &'a str,
    /// アルバムアーティスト(並び替え用)
    pub album_artist_order: &'a str,
    /// 作曲者(並び替え用)
    pub composer_order: &'a str,
    /// ジャンル(並び替え用)
    pub genre_order: &'a str,
}
