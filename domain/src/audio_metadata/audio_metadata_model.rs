use chrono::NaiveDate;

/// オーディオファイルのメタデータのうち、Murack が利用する部分
#[derive(Debug, PartialEq)]
pub struct AudioMetaData {
    /// 曲の再生時間
    pub duration: u32,

    /// 曲名
    pub title: Option<String>,

    /// アーティスト
    pub artist: Option<String>,
    /// アルバム
    pub album: Option<String>,
    /// ジャンル
    pub genre: Option<String>,
    /// アルバムアーティスト
    pub album_artist: Option<String>,
    /// 作曲者
    pub composer: Option<String>,

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
    pub memo: Option<String>,

    /// アートワーク
    pub artworks: Vec<TrackArtwork>,
}

/// 曲へ紐付けるアートワークの情報
#[derive(Debug, PartialEq, Clone)]
pub struct TrackArtwork {
    /// 画像のバイトデータ
    pub image: Vec<u8>,

    /// 画像データのMIMEタイプ
    pub mime_type: String,

    /// 画像タイプ
    ///
    /// FLACやID3で定義された、0〜20の値
    pub picture_type: u8,

    /// 画像の説明
    pub description: String,
}
