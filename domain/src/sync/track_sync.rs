use chrono::NaiveDate;
use murack_core_media::audio_meta::{AudioMetaDataEntry, AudioPictureEntry};

use crate::{artwork::TrackArtwork, string_order_cnv};

/// PC・DB間で同期するべき曲の情報
#[derive(Debug, PartialEq)]
pub struct TrackSync {
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

    /// 歌詞
    pub lyrics: Option<String>,

    /// アートワーク
    pub artworks: Vec<TrackArtwork>,
}

impl TrackSync {
    /// ソート用の曲名
    pub fn title_order(&self) -> Option<String> {
        self.title.as_ref().map(|s| string_order_cnv::cnv(s))
    }

    /// ソート用のアーティスト
    pub fn artist_order(&self) -> Option<String> {
        self.artist.as_ref().map(|s| string_order_cnv::cnv(s))
    }

    /// ソート用のアルバムアーティスト
    pub fn album_artist_order(&self) -> Option<String> {
        self.album_artist.as_ref().map(|s| string_order_cnv::cnv(s))
    }

    /// ソート用のアルバム
    pub fn album_order(&self) -> Option<String> {
        self.album.as_ref().map(|s| string_order_cnv::cnv(s))
    }

    /// ソート用のジャンル
    pub fn genre_order(&self) -> Option<String> {
        self.genre.as_ref().map(|s| string_order_cnv::cnv(s))
    }

    /// ソート用の作曲者
    pub fn composer_order(&self) -> Option<String> {
        self.composer.as_ref().map(|s| string_order_cnv::cnv(s))
    }
}

impl TrackSync {
    /// AudioMetaDataの登録用データに変換
    pub fn get_audio_metadata_entry(&self) -> (AudioMetaDataEntry, Vec<AudioPictureEntry>) {
        (
            AudioMetaDataEntry {
                title: self.title.as_deref(),
                artist: self.artist.as_deref(),
                album: self.album.as_deref(),
                genre: self.genre.as_deref(),
                album_artist: self.album_artist.as_deref(),
                composer: self.composer.as_deref(),
                track_number: self.track_number,
                track_max: self.track_max,
                disc_number: self.disc_number,
                disc_max: self.disc_max,
                release_date: self.release_date,
                memo: self.memo.as_deref(),
            },
            self.artworks.iter().map(AudioPictureEntry::from).collect(),
        )
    }
}
