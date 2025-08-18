use chrono::NaiveDate;
use murack_core_domain::{string_order_cnv, track::TrackDuration};

/// PC・DB間で同期する、曲のメタデータ
#[derive(Debug, PartialEq, Clone)]
pub struct AudioMetadata {
    /// 曲の再生時間
    pub duration: TrackDuration,

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

    /// アートワーク
    pub artworks: Vec<TrackArtwork>,
}

impl AudioMetadata {
    /// ソート用の曲名
    pub fn title_order(&self) -> String {
        string_order_cnv::cnv(&self.title)
    }

    /// ソート用のアーティスト
    pub fn artist_order(&self) -> String {
        string_order_cnv::cnv(&self.artist)
    }

    /// ソート用のアルバムアーティスト
    pub fn album_artist_order(&self) -> String {
        string_order_cnv::cnv(&self.album_artist)
    }

    /// ソート用のアルバム
    pub fn album_order(&self) -> String {
        string_order_cnv::cnv(&self.album)
    }

    /// ソート用のジャンル
    pub fn genre_order(&self) -> String {
        string_order_cnv::cnv(&self.genre)
    }

    /// ソート用の作曲者
    pub fn composer_order(&self) -> String {
        string_order_cnv::cnv(&self.composer)
    }
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
