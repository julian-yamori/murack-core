#[cfg(test)]
mod tests;

use anyhow::Result;
use chrono::NaiveDate;
use sqlx::PgTransaction;

use super::esc::escs;
use crate::{
    db_utils::like_esc,
    filter::{
        ArtworkFilterRange, BoolFilterRange, DateFilterRange, FilterTarget, GroupOperand,
        IntFilterRange, RootFilter, StringFilterRange, TagsFilterRange,
    },
};

/// フィルタを使用して曲 ID を列挙
/// # Arguments
/// - filter: 検索に使用するフィルタ情報
pub async fn get_track_ids<'c>(
    tx: &mut PgTransaction<'c>,
    filter: &RootFilter,
) -> Result<Vec<i32>> {
    let mut query_base = "SELECT tracks.id FROM tracks".to_owned();

    //フィルタから条件を取得して追加
    let query_where = get_query_filter_where(filter);
    if !query_where.is_empty() {
        query_base = format!("{query_base} WHERE {query_where}");
    }

    let list = sqlx::query_scalar(&query_base).fetch_all(&mut **tx).await?;

    Ok(list)
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得
/// # Arguments
/// - filter: 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where(filter_target: &FilterTarget) -> String {
    match filter_target {
        FilterTarget::FilterGroup { op, children } => get_query_filter_where_group(op, children),
        FilterTarget::Tags { range } => get_query_filter_where_tag(range),
        FilterTarget::Rating { range } => get_query_filter_where_int("rating", range),
        FilterTarget::Genre { range } => get_query_filter_where_str("genre", range),
        FilterTarget::Artist { range } => get_query_filter_where_str("artist", range),
        FilterTarget::Albumartist { range } => get_query_filter_where_str("album_artist", range),
        FilterTarget::Album { range } => get_query_filter_where_str("album", range),
        FilterTarget::Composer { range } => get_query_filter_where_str("composer", range),
        FilterTarget::Title { range } => get_query_filter_where_str("title", range),
        FilterTarget::Artwork { range } => get_query_filter_where_artwork(range),
        FilterTarget::Duration { range } => get_query_filter_where_int("duration", range),
        FilterTarget::ReleaseDate { range } => get_query_filter_where_date("release_date", range),
        FilterTarget::TrackNumber { range } => get_query_filter_where_int("track_number", range),
        FilterTarget::TrackMax { range } => get_query_filter_where_int("track_max", range),
        FilterTarget::DiscNumber { range } => get_query_filter_where_int("disc_number", range),
        FilterTarget::DiscMax { range } => get_query_filter_where_int("disc_max", range),
        FilterTarget::Memo { range } => get_query_filter_where_str("memo", range),
        FilterTarget::MemoManage { range } => get_query_filter_where_str("memo_manage", range),

        // TODO DateTime との比較は危なそう #17
        FilterTarget::EntryDate { range } => get_query_filter_where_date("created_at", range),

        FilterTarget::OriginalTrack { range } => {
            get_query_filter_where_str("original_track", range)
        }
        FilterTarget::SuggestTarget { range } => {
            get_query_filter_where_bool("suggest_target", range)
        }
    }
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(グループ用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_group(op: &GroupOperand, children: &[FilterTarget]) -> String {
    //各フィルタのクエリを連結
    let ope = match op {
        GroupOperand::And => " and ",
        GroupOperand::Or => " or ",
    };

    let combined_query = children
        .iter()
        .map(get_query_filter_where)
        .collect::<Vec<String>>()
        .join(ope);

    if combined_query.is_empty() {
        return combined_query;
    }

    //クエリ文字列は()で囲む
    format!("({combined_query})")
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(文字列用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_str(column_name: &str, range: &StringFilterRange) -> String {
    //演算子〜値の左側、値の右側、LIKE区を使うか
    //範囲指定により分岐
    let (left, right, use_like, filter_value) = match range {
        //指定文字列と等しい
        StringFilterRange::Equal { value } => (" = ", "", false, value),
        //指定文字列と等しくない
        StringFilterRange::NotEqual { value } => (" != ", "", false, value),
        //指定文字列を含む
        StringFilterRange::Contain { value } => (" like '%' || ", " || '%'", true, value),
        //指定文字列を含まない
        StringFilterRange::NotContain { value } => (" not like '%' || ", " || '%'", true, value),
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

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(整数用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_int(clm_name: &str, range: &IntFilterRange) -> String {
    match range {
        //指定値と等しい
        IntFilterRange::Equal { value } => format!("{clm_name} = {value}"),
        //指定値と等しくない
        //※nullは含めない仕様(WalkBase1がそうなっていたので)
        IntFilterRange::NotEqual { value } => format!("{clm_name} <> {value}"),
        //指定値以上
        IntFilterRange::LargeEqual { value } => format!("{clm_name} >= {value}"),
        //指定値以下
        IntFilterRange::SmallEqual { value } => format!("{clm_name} <= {value}"),
        //指定範囲内
        IntFilterRange::RangeIn { min, max } => {
            let (small, large) = get_ordered_int(*min, *max);
            format!("({clm_name} >= {small} and {clm_name} <= {large})")
        }
        //指定範囲外
        IntFilterRange::RangeOut { min, max } => {
            let (small, large) = get_ordered_int(*min, *max);
            format!("({clm_name} < {small} or {clm_name} > {large})")
        }
    }
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(タグ用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_tag(range: &TagsFilterRange) -> String {
    //タグで検索するクエリを取得する関数
    fn get_query_where_by_tag(tag_id: i32) -> String {
        format!(
            "EXISTS(SELECT * FROM track_tags AS t WHERE t.track_id = tracks.id AND t.tag_id = {tag_id})"
        )
    }

    match range {
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

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(フラグ用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_bool(clm_name: &str, range: &BoolFilterRange) -> String {
    match range {
        //true
        BoolFilterRange::True => format!("{clm_name} = true"),
        //false
        BoolFilterRange::False => format!("{clm_name} = false"),
    }
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(アートワーク用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_artwork(range: &ArtworkFilterRange) -> String {
    //存在すればtrueのsql
    let base_sql = "EXISTS(SELECT * FROM track_artworks AS a WHERE a.track_id = tracks.id)";

    match range {
        //アートワーク：ある
        ArtworkFilterRange::Has => base_sql.to_owned(),
        //アートワーク：ない
        ArtworkFilterRange::None => format!("NOT {base_sql}"),
    }
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(日付用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_date(clm_name: &str, range: &DateFilterRange) -> String {
    match range {
        //指定値と等しい
        DateFilterRange::Equal { value } => {
            format!("{} = {}", clm_name, escs(&date_to_str(value)))
        }
        //指定値と等しくない
        //※nullは含めない仕様(WalkBase1がそうなっていたので)
        DateFilterRange::NotEqual { value } => {
            format!("{} <> {}", clm_name, escs(&date_to_str(value)))
        }
        //指定値以前
        DateFilterRange::Before { value } => {
            format!("{} <= {}", clm_name, escs(&date_to_str(value)))
        }
        //指定値以後
        DateFilterRange::After { value } => {
            format!("{} >= {}", clm_name, escs(&date_to_str(value)))
        }
        //なし
        DateFilterRange::None => format!("{clm_name} is null"),
    }
}

fn date_to_str(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
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
