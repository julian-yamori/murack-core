use chrono::NaiveDate;
use murack_core_domain::string_order_cnv;

use crate::audio_metadata::TrackArtwork;

/// PC・DB間で同期するべき曲の情報
#[derive(Debug, PartialEq, Clone)]
pub struct TrackSync {
    /// 曲の再生時間
    pub duration: u32,

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

impl TrackSync {
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
