use num_derive::FromPrimitive;
use std::fmt;

/// フィルタ種類
#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive)]
pub enum FilterTarget {
    /// 集合フィルタ
    FilterGroup = 0,
    /// タグ
    Tags = 1,
    /// 曲のレート(評価)
    Rating = 2,
    /// ジャンル
    Genre = 3,
    /// アーティスト
    Artist = 4,
    /// アルバムアーティスト
    Albumartist = 5,
    /// アルバム
    Album = 6,
    /// 作曲者
    Composer = 7,
    /// 曲名
    Title = 8,
    /// 曲のアートワーク
    Artwork = 9,
    /// 再生時間
    Duration = 10,
    /// リリース日
    ReleaseDate = 11,
    /// トラック番号
    TrackNumber = 12,
    /// トラック最大数
    TrackMax = 13,
    /// ディスク番号
    DiscNumber = 14,
    /// ディスク最大数
    DiscMax = 15,
    /// メモ
    Memo = 16,
    /// 管理メモ
    MemoManage = 17,
    /// 登録日
    EntryDate = 18,
    /// 原曲
    OriginalSong = 19,
    /// サジェスト対象
    SuggestTarget = 20,
}

/// フィルタリング値の範囲指定タイプ
#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive)]
pub enum FilterValueRange {
    /// グループ用：AND
    GroupAnd = 0,
    /// グループ用：OR
    GroupOr = 1,

    /// 文字列：指定文字列と等しい
    StrEqual = 2,
    /// 文字列：指定文字列と等しくない
    StrNotEqual = 3,
    /// 文字列：指定文字列を含む
    StrContain = 4,
    /// 文字列：指定文字列を含まない
    StrNotContain = 5,
    /// 文字列：指定文字列から始まる
    StrStart = 6,
    /// 文字列：指定文字列で終わる
    StrEnd = 7,

    /// 数値：指定値と等しい
    IntEqual = 8,
    /// 数値：指定値と等しくない
    IntNotEqual = 9,
    /// 数値：指定値以上
    IntLargeEqual = 10,
    /// 数値：指定値以下
    IntSmallEqual = 11,
    /// 数値：指定範囲内
    IntRangeIn = 12,
    /// 数値：指定範囲外
    IntRangeOut = 13,

    /// ID：有効(-1以外)
    IdValid = 14,
    /// ID：無効(-1)
    IdInvalid = 15,

    /// タグ：含む
    TagContain = 16,
    /// タグ：含まない
    TagNotContain = 17,
    /// タグ：タグを持たない
    TagNone = 18,

    /// フラグ：true
    BoolTrue = 19,
    /// フラグ：false
    BoolFalse = 20,

    /// アートワーク：ある
    ArtworkHas = 21,
    /// アートワーク：ない
    ArtworkNone = 22,

    /// 日付：指定値と等しい
    DateEqual = 23,
    /// 日付：指定値と等しくない
    DateNotEqual = 24,
    /// 日付：指定値以前
    DateBefore = 25,
    /// 日付：指定値以後
    DateAfter = 26,
    /// 日付：ない
    DateNone = 27,
}

/// フィルタリング対象値の型
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FilterValueType {
    /// フィルタグループ
    FilterGroup,
    /// 文字列
    String,
    /// 数値
    Int,
    /// ID
    Id,
    /// タグ
    Tags,
    /// フラグ
    Bool,
    /// アートワーク
    Artwork,
    /// 日付
    Date,
}

impl FilterTarget {
    /// フィルタ種類に応じた値の型を取得
    pub fn value_type(&self) -> FilterValueType {
        match self {
            FilterTarget::FilterGroup => FilterValueType::FilterGroup,
            FilterTarget::Title => FilterValueType::String,
            FilterTarget::Artist => FilterValueType::String,
            FilterTarget::Album => FilterValueType::String,
            FilterTarget::Genre => FilterValueType::String,
            FilterTarget::Albumartist => FilterValueType::String,
            FilterTarget::Composer => FilterValueType::String,
            FilterTarget::TrackNumber => FilterValueType::Int,
            FilterTarget::TrackMax => FilterValueType::Int,
            FilterTarget::DiscNumber => FilterValueType::Int,
            FilterTarget::DiscMax => FilterValueType::Int,
            FilterTarget::ReleaseDate => FilterValueType::Date,
            FilterTarget::Rating => FilterValueType::Int,
            FilterTarget::Duration => FilterValueType::Int,
            FilterTarget::Tags => FilterValueType::Tags,
            FilterTarget::Artwork => FilterValueType::Artwork,
            FilterTarget::Memo => FilterValueType::String,
            FilterTarget::MemoManage => FilterValueType::String,
            FilterTarget::EntryDate => FilterValueType::Date,
            FilterTarget::OriginalSong => FilterValueType::String,
            FilterTarget::SuggestTarget => FilterValueType::Bool,
        }
    }
}

impl fmt::Display for FilterTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            FilterTarget::FilterGroup => "親フィルタ",
            FilterTarget::Title => "曲名",
            FilterTarget::Artist => "アーティスト",
            FilterTarget::Album => "アルバム",
            FilterTarget::Genre => "ジャンル",
            FilterTarget::Albumartist => "アルバムアーティスト",
            FilterTarget::Composer => "作曲者",
            FilterTarget::TrackNumber => "トラック番号",
            FilterTarget::TrackMax => "トラック番号最大",
            FilterTarget::DiscNumber => "ディスク番号",
            FilterTarget::DiscMax => "ディスク番号最大",
            FilterTarget::ReleaseDate => "リリース日",
            FilterTarget::Rating => "レート",
            FilterTarget::Duration => "再生時間",
            FilterTarget::Tags => "タグ",
            FilterTarget::Artwork => "アートワーク",
            FilterTarget::Memo => "メモ",
            FilterTarget::MemoManage => "管理メモ",
            FilterTarget::EntryDate => "登録日",
            FilterTarget::OriginalSong => "原曲",
            FilterTarget::SuggestTarget => "サジェスト対象",
        };
        f.write_str(s)
    }
}

impl fmt::Display for FilterValueRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            FilterValueRange::GroupAnd => "GroupAnd",
            FilterValueRange::GroupOr => "GroupOr",
            FilterValueRange::StrEqual => "StrEqual",
            FilterValueRange::StrNotEqual => "StrNotEqual",
            FilterValueRange::StrContain => "StrContain",
            FilterValueRange::StrNotContain => "StrNotContain",
            FilterValueRange::StrStart => "StrStart",
            FilterValueRange::StrEnd => "StrEnd",
            FilterValueRange::IntEqual => "IntEqual",
            FilterValueRange::IntNotEqual => "IntNotEqual",
            FilterValueRange::IntLargeEqual => "IntLargeEqual",
            FilterValueRange::IntSmallEqual => "IntSmallEqual",
            FilterValueRange::IntRangeIn => "IntRangeIn",
            FilterValueRange::IntRangeOut => "IntRangeOut",
            FilterValueRange::IdValid => "IdValid",
            FilterValueRange::IdInvalid => "IdInvalid",
            FilterValueRange::TagContain => "TagContain",
            FilterValueRange::TagNotContain => "TagNotContain",
            FilterValueRange::TagNone => "TagNone",
            FilterValueRange::BoolTrue => "BoolTrue",
            FilterValueRange::BoolFalse => "BoolFalse",
            FilterValueRange::ArtworkHas => "ArtworkHas",
            FilterValueRange::ArtworkNone => "ArtworkNone",
            FilterValueRange::DateEqual => "DateEqual",
            FilterValueRange::DateNotEqual => "DateNotEqual",
            FilterValueRange::DateBefore => "DateBefore",
            FilterValueRange::DateAfter => "DateAfter",
            FilterValueRange::DateNone => "DateNone",
        };
        f.write_str(s)
    }
}
