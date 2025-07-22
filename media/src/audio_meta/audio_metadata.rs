use super::{AudioPicture, AudioPictureEntry};
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
    pub artworks: Vec<AudioPicture>,
}

/// オーディオファイルのメタデータの登録用データ
///
/// アートワークはEntryをスライスで扱いたい都合上、別定義
#[derive(Debug, PartialEq)]
pub struct AudioMetaDataEntry<'a> {
    /// 曲名
    pub title: Option<&'a str>,

    /// アーティスト
    pub artist: Option<&'a str>,
    /// アルバム
    pub album: Option<&'a str>,
    /// ジャンル
    pub genre: Option<&'a str>,
    /// アルバムアーティスト
    pub album_artist: Option<&'a str>,
    /// 作曲者
    pub composer: Option<&'a str>,

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
    pub memo: Option<&'a str>,
}

impl AudioMetaData {
    /// 登録用データに変換
    pub fn get_entry(&self) -> (AudioMetaDataEntry, Vec<AudioPictureEntry>) {
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
