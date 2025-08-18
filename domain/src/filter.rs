#[cfg(test)]
mod tests;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::db_utils::{escs, like_esc};

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
    /// `tracks.duration` と同じく、IntFilterRange の値にはミリ秒の i32 を格納する。
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

impl FilterTarget {
    /// SQL の WHERE で使用する条件式に変換
    pub fn where_expression(&self) -> String {
        match self {
            FilterTarget::FilterGroup { op, children } => group_where_expression(op, children),
            FilterTarget::Tags { range } => range.where_expression(),
            FilterTarget::Rating { range } => range.where_expression("rating"),
            FilterTarget::Genre { range } => range.where_expression("genre"),
            FilterTarget::Artist { range } => range.where_expression("artist"),
            FilterTarget::Albumartist { range } => range.where_expression("album_artist"),
            FilterTarget::Album { range } => range.where_expression("album"),
            FilterTarget::Composer { range } => range.where_expression("composer"),
            FilterTarget::Title { range } => range.where_expression("title"),
            FilterTarget::Artwork { range } => range.where_expression(),
            FilterTarget::Duration { range } => range.where_expression("duration"),
            FilterTarget::ReleaseDate { range } => range.where_expression("release_date"),
            FilterTarget::TrackNumber { range } => range.where_expression("track_number"),
            FilterTarget::TrackMax { range } => range.where_expression("track_max"),
            FilterTarget::DiscNumber { range } => range.where_expression("disc_number"),
            FilterTarget::DiscMax { range } => range.where_expression("disc_max"),
            FilterTarget::Memo { range } => range.where_expression("memo"),
            FilterTarget::MemoManage { range } => range.where_expression("memo_manage"),

            // TODO DateTime との比較は危なそう #17
            FilterTarget::EntryDate { range } => range.where_expression("created_at"),

            FilterTarget::OriginalTrack { range } => range.where_expression("original_track"),
            FilterTarget::SuggestTarget { range } => range.where_expression("suggest_target"),
        }
    }
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

/// `FilterTarget::FilterGroup` を、SQL の WHERE で使用する条件式に変換
pub fn group_where_expression(op: &GroupOperand, children: &[FilterTarget]) -> String {
    //各フィルタのクエリを連結
    let ope = match op {
        GroupOperand::And => " and ",
        GroupOperand::Or => " or ",
    };

    let combined_query = children
        .iter()
        .map(FilterTarget::where_expression)
        .collect::<Vec<String>>()
        .join(ope);

    if combined_query.is_empty() {
        return combined_query;
    }

    //クエリ文字列は()で囲む
    format!("({combined_query})")
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

impl StringFilterRange {
    /// SQL の WHERE で使用する条件式に変換
    pub fn where_expression(&self, column_name: &str) -> String {
        //演算子〜値の左側、値の右側、LIKE区を使うか
        //範囲指定により分岐
        let (left, right, use_like, filter_value) = match self {
            //指定文字列と等しい
            StringFilterRange::Equal { value } => (" = ", "", false, value),
            //指定文字列と等しくない
            StringFilterRange::NotEqual { value } => (" != ", "", false, value),
            //指定文字列を含む
            StringFilterRange::Contain { value } => (" like '%' || ", " || '%'", true, value),
            //指定文字列を含まない
            StringFilterRange::NotContain { value } => {
                (" not like '%' || ", " || '%'", true, value)
            }
            //指定文字列から始まる
            StringFilterRange::Start { value } => (" like ", " || '%'", true, value),
            //指定文字列で終わる
            StringFilterRange::End { value } => (" like '%' || ", "", true, value),
        };
        let mut r_string = right.to_owned();

        //必要ならlike文のエスケープ処理
        let cmp_value = if use_like && like_esc::is_need(filter_value) {
            r_string = format!("{r_string} escape '$'");
            like_esc::escape(filter_value)
        } else {
            filter_value.to_owned()
        };

        format!("{column_name}{left}{}{r_string}", escs(&cmp_value))
    }
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

impl IntFilterRange {
    /// SQL の WHERE で使用する条件式に変換
    pub fn where_expression(&self, column_name: &str) -> String {
        match self {
            //指定値と等しい
            IntFilterRange::Equal { value } => format!("{column_name} = {value}"),
            //指定値と等しくない
            //※nullは含めない仕様(WalkBase1がそうなっていたので)
            IntFilterRange::NotEqual { value } => format!("{column_name} <> {value}"),
            //指定値以上
            IntFilterRange::LargeEqual { value } => format!("{column_name} >= {value}"),
            //指定値以下
            IntFilterRange::SmallEqual { value } => format!("{column_name} <= {value}"),
            //指定範囲内
            IntFilterRange::RangeIn { min, max } => {
                let (small, large) = get_ordered_int(*min, *max);
                format!("({column_name} >= {small} and {column_name} <= {large})")
            }
            //指定範囲外
            IntFilterRange::RangeOut { min, max } => {
                let (small, large) = get_ordered_int(*min, *max);
                format!("({column_name} < {small} or {column_name} > {large})")
            }
        }
    }
}

/// IntFilterRange の min と max を念のため大小比較
/// # Returns
/// - .0: 小さい方の値
/// - .1: 大きい方の値
fn get_ordered_int(val1: i32, val2: i32) -> (i32, i32) {
    if val1 <= val2 {
        (val1, val2)
    } else {
        (val2, val1)
    }
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

impl TagsFilterRange {
    /// SQL の WHERE で使用する条件式に変換
    pub fn where_expression(&self) -> String {
        //タグで検索するクエリを取得する関数
        fn get_query_where_by_tag(tag_id: i32) -> String {
            format!(
                "EXISTS(SELECT * FROM track_tags AS t WHERE t.track_id = tracks.id AND t.tag_id = {tag_id})"
            )
        }

        match self {
            //タグ：含む
            TagsFilterRange::Contain { value } => get_query_where_by_tag(*value),
            //タグ：含まない
            TagsFilterRange::NotContain { value } => {
                format!("NOT {}", get_query_where_by_tag(*value))
            }
            //タグ：タグを持たない
            TagsFilterRange::None => {
                "NOT EXISTS(SELECT * FROM track_tags AS t WHERE t.track_id = tracks.id)".to_owned()
            }
        }
    }
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

impl BoolFilterRange {
    /// SQL の WHERE で使用する条件式に変換
    pub fn where_expression(&self, column_name: &str) -> String {
        match self {
            //true
            BoolFilterRange::True => format!("{column_name} = true"),
            //false
            BoolFilterRange::False => format!("{column_name} = false"),
        }
    }
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

impl ArtworkFilterRange {
    /// SQL の WHERE で使用する条件式に変換
    pub fn where_expression(&self) -> String {
        //存在すればtrueのsql
        let base_sql = "EXISTS(SELECT * FROM track_artworks AS a WHERE a.track_id = tracks.id)";

        match self {
            //アートワーク：ある
            ArtworkFilterRange::Has => base_sql.to_owned(),
            //アートワーク：ない
            ArtworkFilterRange::None => format!("NOT {base_sql}"),
        }
    }
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

impl DateFilterRange {
    /// SQL の WHERE で使用する条件式に変換
    pub fn where_expression(&self, column_name: &str) -> String {
        match self {
            //指定値と等しい
            DateFilterRange::Equal { value } => {
                format!("{} = {}", column_name, escs(&date_to_str(value)))
            }
            //指定値と等しくない
            //※nullは含めない仕様(WalkBase1がそうなっていたので)
            DateFilterRange::NotEqual { value } => {
                format!("{} <> {}", column_name, escs(&date_to_str(value)))
            }
            //指定値以前
            DateFilterRange::Before { value } => {
                format!("{} <= {}", column_name, escs(&date_to_str(value)))
            }
            //指定値以後
            DateFilterRange::After { value } => {
                format!("{} >= {}", column_name, escs(&date_to_str(value)))
            }
            //なし
            DateFilterRange::None => format!("{column_name} is null"),
        }
    }
}

fn date_to_str(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}
