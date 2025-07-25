use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 最上位フィルタ (GUI では、最上位は FilterGroup である必要がある)
pub type RootFilter = FilterTarget;

/// フィルタの対象の項目と、その項目に対応した条件情報
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "target")]
pub enum FilterTarget {
    /// 集合フィルタ
    #[serde(rename = "group")]
    FilterGroup {
        op: GroupOperand,
        children: Vec<FilterTarget>,
    },

    /// タグ (track_tags.tag_id)
    #[serde(rename = "tags")]
    Tags { range: TagsFilterRange },

    /// レート (rating)
    #[serde(rename = "rating")]
    Rating { range: IntFilterRange },

    /// ジャンル (genre)
    #[serde(rename = "genre")]
    Genre { range: StringFilterRange },

    /// アーティスト (artist)
    #[serde(rename = "artist")]
    Artist { range: StringFilterRange },

    /// アルバムアーティスト (album_artist)
    #[serde(rename = "albumartist")]
    Albumartist { range: StringFilterRange },

    /// アルバム (album)
    #[serde(rename = "album")]
    Album { range: StringFilterRange },

    /// 作曲者 (composer)
    #[serde(rename = "composer")]
    Composer { range: StringFilterRange },

    /// 曲名 (title)
    #[serde(rename = "title")]
    Title { range: StringFilterRange },

    /// アートワーク (track_artworkテーブル)
    #[serde(rename = "artwork")]
    Artwork { range: ArtworkFilterRange },

    /// 再生時間 (duration)
    ///
    /// 画面からの入力は「分:秒」形式
    #[serde(rename = "duration")]
    Duration { range: IntFilterRange },

    /// リリース日 (release_date)
    #[serde(rename = "release_date")]
    ReleaseDate { range: DateFilterRange },

    /// トラック番号 (track_number)
    #[serde(rename = "track_number")]
    TrackNumber { range: IntFilterRange },

    /// トラック最大数 (track_max)
    #[serde(rename = "track_max")]
    TrackMax { range: IntFilterRange },

    /// ディスク番号 (disc_number)
    #[serde(rename = "disc_number")]
    DiscNumber { range: IntFilterRange },

    /// ディスク最大数 (disc_max)
    #[serde(rename = "disc_max")]
    DiscMax { range: IntFilterRange },

    /// メモ (memo)
    #[serde(rename = "memo")]
    Memo { range: StringFilterRange },

    /// 管理メモ (memo_manage)
    #[serde(rename = "memo_manage")]
    MemoManage { range: StringFilterRange },

    /// 登録日 (entry_date)
    #[serde(rename = "entry_date")]
    EntryDate { range: DateFilterRange },

    /// 原曲 (original_track)
    #[serde(rename = "original_track")]
    OriginalTrack { range: StringFilterRange },

    /// サジェスト対象 (suggest_target)
    #[serde(rename = "suggest_target")]
    SuggestTarget { range: BoolFilterRange },
}

/// 集合フィルタの条件指定方法
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GroupOperand {
    /// 全てを満たす
    #[serde(rename = "and")]
    And,

    /// いずれかを満たす
    #[serde(rename = "or")]
    Or,
}

/// 文字列で絞り込み
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum StringFilterRange {
    /// 指定文字列と等しい
    #[serde(rename = "equal")]
    Equal { value: String },

    /// 指定文字列と異なる
    #[serde(rename = "not_equal")]
    NotEqual { value: String },

    /// 指定文字列を含む
    #[serde(rename = "contain")]
    Contain { value: String },

    /// 指定文字列を含まない
    #[serde(rename = "not_contain")]
    NotContain { value: String },

    /// 指定文字列から始まる
    #[serde(rename = "start")]
    Start { value: String },

    /// 指定文字列で終わる
    #[serde(rename = "end")]
    End { value: String },
}

/// 数値で絞り込み
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum IntFilterRange {
    /// 指定値と等しい
    #[serde(rename = "equal")]
    Equal { value: i32 },

    /// 指定値と異なる
    #[serde(rename = "not_equal")]
    NotEqual { value: i32 },

    /// 指定値以上
    #[serde(rename = "large_equal")]
    LargeEqual { value: i32 },

    /// 指定値以下
    #[serde(rename = "small_equal")]
    SmallEqual { value: i32 },

    /// 指定範囲内
    #[serde(rename = "range_in")]
    RangeIn { min: i32, max: i32 },

    /// 指定範囲外
    #[serde(rename = "range_out")]
    RangeOut { min: i32, max: i32 },
}

/// タグで絞り込み
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum TagsFilterRange {
    /// 指定されたタグを含む
    #[serde(rename = "contain")]
    Contain { value: i32 },
    /// 指定されたタグを含まない
    #[serde(rename = "not_contain")]
    NotContain { value: i32 },
    /// タグを持たない
    #[serde(rename = "none")]
    None,
}

/// bool で絞り込み
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum BoolFilterRange {
    /// フラグが true
    #[serde(rename = "true")]
    True,

    /// フラグが false
    #[serde(rename = "false")]
    False,
}

/// アートワークで絞り込み
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum ArtworkFilterRange {
    /// アートワークがある
    #[serde(rename = "has")]
    Has,

    /// アートワークが無い
    #[serde(rename = "none")]
    None,
}

/// 日付で絞り込み
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum DateFilterRange {
    /// 日付：指定値と等しい
    #[serde(rename = "equal")]
    Equal { value: NaiveDate },

    /// 日付：指定値と異なる
    #[serde(rename = "not_equal")]
    NotEqual { value: NaiveDate },

    /// 日付：指定値以前
    #[serde(rename = "before")]
    Before { value: NaiveDate },

    /// 日付：指定値以後
    #[serde(rename = "after")]
    After { value: NaiveDate },

    /// 日付：ない
    #[serde(rename = "none")]
    None,
}
