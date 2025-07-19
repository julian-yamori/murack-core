use chrono::{DateTime, NaiveDate, Utc};

use crate::converts::DbOptionString;

/// songテーブルのレコード
#[derive(Debug, PartialEq, Clone)]
pub struct SongRow {
    /// 曲ID
    pub id: i32,

    /// 曲の長さ(ミリ秒)
    pub duration: i32,

    /// 曲ファイルのパス
    pub path: String,
    /// フォルダID
    pub folder_id: Option<i32>,

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
    pub release_date: Option<NaiveDate>,

    /// メモ
    pub memo: DbOptionString,

    /// レート
    pub rating: i16,
    /// 原曲
    pub original_track: DbOptionString,
    /// サジェスト対象フラグ
    pub suggest_target: bool,
    /// 管理メモ
    pub memo_manage: DbOptionString,

    /// 歌詞
    pub lyrics: DbOptionString,

    /// 曲名(並び替え用)
    pub title_order: String,
    /// アーティスト(並び替え用)
    pub artist_order: String,
    /// アルバム(並び替え用)
    pub album_order: String,
    /// ジャンル(並び替え用)
    pub genre_order: String,
    /// アルバムアーティスト(並び替え用)
    pub album_artist_order: String,
    /// 作曲者(並び替え用)
    pub composer_order: String,

    /// 登録日
    pub created_at: DateTime<Utc>,
}
