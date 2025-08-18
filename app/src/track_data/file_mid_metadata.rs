use std::time::Duration;

use chrono::NaiveDate;

use crate::track_data::{AudioMetadata, TrackArtwork};

/// ファイルと相互変換しやすい形式の曲データ
///
/// フォーマット毎に定義を分けた方がいい気もするけど…… (FLAC は `Vec<String>` の方がいい)
/// ひとまず旧処理互換。
#[derive(Debug, PartialEq)]
pub struct FileMidMetadata {
    /// 曲の再生時間
    pub duration: Duration,

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

impl FileMidMetadata {
    /// AudioMetadata から、FileMidMetadata と歌詞に変換
    pub fn from_audio_metadata(value: AudioMetadata) -> (Self, String) {
        (
            FileMidMetadata {
                duration: value.duration.into(),
                title: none_if_empty(value.title),
                artist: none_if_empty(value.artist),
                album: none_if_empty(value.album),
                genre: none_if_empty(value.genre),
                album_artist: none_if_empty(value.album_artist),
                composer: none_if_empty(value.composer),
                track_number: value.track_number,
                track_max: value.track_max,
                disc_number: value.disc_number,
                disc_max: value.disc_max,
                release_date: value.release_date,
                memo: none_if_empty(value.memo),
                artworks: value.artworks,
            },
            value.lyrics,
        )
    }
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}
