#[cfg(test)]
mod tests;

use super::esc::escs;
use crate::{Error, like_esc, sql_func};
use anyhow::Result;
use domain::{
    db_wrapper::TransactionWrapper,
    filter::{Filter, FilterTarget, FilterValueRange, FilterValueType},
};
use mockall::automock;
use std::str::FromStr;

/// フィルタを使用した曲データ列挙
#[automock]
pub trait SongListerFilter {
    /// 曲IDを列挙
    /// # Arguments
    /// - filter: 検索に使用するフィルタ情報
    fn list_song_id<'c>(&self, tx: &TransactionWrapper<'c>, filter: &Filter) -> Result<Vec<i32>>;
}

/// SongListerFilterの本実装
pub struct SongListerFilterImpl {}

impl SongListerFilter for SongListerFilterImpl {
    /// 曲IDを列挙
    /// # Arguments
    /// - filter: 検索に使用するフィルタ情報
    fn list_song_id<'c>(&self, tx: &TransactionWrapper<'c>, filter: &Filter) -> Result<Vec<i32>> {
        let mut query_base = "select [song].[rowid] from [song]".to_owned();

        //フィルタから条件を取得して追加
        let query_where = get_query_filter_where(filter)?;
        if !query_where.is_empty() {
            query_base = format!("{query_base} where {query_where}");
        }

        sql_func::select_list(tx, &query_base, [], |row| row.get(0))
    }
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得
/// # Arguments
/// - filter: 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where(filter: &Filter) -> Result<String> {
    Ok(match filter.target.value_type() {
        //グループ
        FilterValueType::FilterGroup => get_query_filter_where_group(filter)?,
        //文字列
        FilterValueType::String => get_query_filter_where_str(filter)?,
        //数値
        FilterValueType::Int => get_query_filter_where_int(filter)?,
        //ID
        FilterValueType::Id => get_query_filter_where_id(filter)?,
        //タグ
        FilterValueType::Tags => get_query_filter_where_tag(filter)?,
        //フラグ
        FilterValueType::Bool => get_query_filter_where_bool(filter)?,
        //アートワーク
        FilterValueType::Artwork => get_query_filter_where_artwork(filter)?,
        //日付
        FilterValueType::Date => get_query_filter_where_date(filter)?,
    })
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(グループ用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_group(filter: &Filter) -> Result<String> {
    //各フィルタのクエリを連結
    let ope = if filter.range == FilterValueRange::GroupOr {
        " or "
    } else {
        " and "
    };

    let combined_query = filter
        .children
        .iter()
        .map(get_query_filter_where)
        .collect::<Result<Vec<String>>>()?
        .join(ope);

    if combined_query.is_empty() {
        return Ok(combined_query);
    }

    //クエリ文字列は()で囲む
    Ok(format!("({combined_query})"))
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(文字列用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_str(filter: &Filter) -> Result<String> {
    //演算子〜値の左側、値の右側、LIKE区を使うか
    //範囲指定により分岐
    let (left, right, use_like) = match filter.range {
        //指定文字列と等しい
        FilterValueRange::StrEqual => (" = ", "", false),
        //指定文字列と等しくない
        FilterValueRange::StrNotEqual => (" != ", "", false),
        //指定文字列を含む
        FilterValueRange::StrContain => (" like '%' || ", " || '%'", true),
        //指定文字列を含まない
        FilterValueRange::StrNotContain => (" not like '%' || ", " || '%'", true),
        //指定文字列から始まる
        FilterValueRange::StrStart => (" like ", " || '%'", true),
        //指定文字列で終わる
        FilterValueRange::StrEnd => (" like '%' || ", "", true),

        _ => return Err(invalid_filter_range_for_target(filter)),
    };
    let mut r_string = right.to_owned();

    //必要ならlike文のエスケープ処理
    let cmp_value = if use_like && like_esc::is_need(&filter.str_value) {
        r_string = format!("{r_string} escape '$'");
        like_esc::escape(&filter.str_value)
    } else {
        filter.str_value.to_owned()
    };

    Ok(format!(
        "{}{}{}{}",
        target_to_clm_name(filter.target),
        left,
        escs(&cmp_value),
        r_string
    ))
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(整数用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_int(filter: &Filter) -> Result<String> {
    let clm_name = target_to_clm_name(filter.target);

    //第一入力値が未入力の場合
    if filter.str_value.is_empty() {
        //一部範囲は、nullであるかどうかでフィルタリングする
        return Ok(match filter.range {
            //nullと等しい
            FilterValueRange::IntEqual => format!("{clm_name} is null"),
            //nullと等しくない
            FilterValueRange::IntNotEqual => format!("{clm_name} is not null"),
            //それ以外は必ずfalse
            _ => FALSE_QUERY.to_owned(),
        });
    }
    Ok(match filter.range {
        //指定値と等しい
        FilterValueRange::IntEqual => format!("{} = {}", clm_name, filter.str_value),
        //指定値と等しくない
        //※nullは含めない仕様(WalkBase1がそうなっていたので)
        FilterValueRange::IntNotEqual => format!("{} <> {}", clm_name, filter.str_value),
        //指定値以上
        FilterValueRange::IntLargeEqual => format!("{} >= {}", clm_name, filter.str_value),
        //指定値以下
        FilterValueRange::IntSmallEqual => format!("{} <= {}", clm_name, filter.str_value),
        //指定範囲内
        FilterValueRange::IntRangeIn => {
            //第二入力値もチェック
            if filter.str_value2.is_empty() {
                FALSE_QUERY.to_owned()
            } else {
                let (small, large) = get_ordered_int(filter)?;
                format!("({clm_name} >= {small} and {clm_name} <= {large})")
            }
        }
        //指定範囲外
        FilterValueRange::IntRangeOut => {
            //第二入力値もチェック
            if filter.str_value2.is_empty() {
                FALSE_QUERY.to_owned()
            } else {
                let (small, large) = get_ordered_int(filter)?;
                format!("({clm_name} < {small} or {clm_name} > {large})")
            }
        }
        _ => return Err(invalid_filter_range_for_target(filter)),
    })
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(ID用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_id(filter: &Filter) -> Result<String> {
    let clm_name = target_to_clm_name(filter.target);

    Ok(match filter.range {
        //有効(null以外)
        FilterValueRange::IdValid => format!("{clm_name} is not null"),
        //無効(null)
        FilterValueRange::IdInvalid => format!("{clm_name} is null"),
        _ => return Err(invalid_filter_range_for_target(filter)),
    })
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(タグ用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_tag(filter: &Filter) -> Result<String> {
    //空なら、「持たない」以外はfalse
    if filter.str_value.is_empty() && filter.range != FilterValueRange::TagNone {
        return Ok(FALSE_QUERY.to_owned());
    }

    //タグで検索するクエリを取得する関数
    fn get_query_where_by_tag(tag_id: &str) -> String {
        format!(
            "exists(select * from [song_tags] AS t where t.[song_id] = song.[rowid] and t.[tag_id] = {tag_id})"
        )
    }

    Ok(match filter.range {
        //タグ：含む
        FilterValueRange::TagContain => get_query_where_by_tag(&filter.str_value),
        //タグ：含まない
        FilterValueRange::TagNotContain => {
            format!("not {}", get_query_where_by_tag(&filter.str_value))
        }
        //タグ：タグを持たない
        FilterValueRange::TagNone => {
            "not exists(select * from [song_tags] AS t where t.[song_id] = song.[rowid])".to_owned()
        }
        _ => return Err(invalid_filter_range_for_target(filter)),
    })
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(フラグ用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_bool(filter: &Filter) -> Result<String> {
    let clm_name = target_to_clm_name(filter.target);

    Ok(match filter.range {
        //true
        FilterValueRange::BoolTrue => format!("{clm_name} <> 0"),
        //false
        FilterValueRange::BoolFalse => format!("{clm_name} = 0"),
        _ => return Err(invalid_filter_range_for_target(filter)),
    })
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(アートワーク用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_artwork(filter: &Filter) -> Result<String> {
    //存在すればtrueのsql
    let base_sql = "exists(select * from [song_artwork] as a where a.[song_id] = song.[rowid])";

    Ok(match filter.range {
        //アートワーク：ある
        FilterValueRange::ArtworkHas => base_sql.to_owned(),
        //アートワーク：ない
        FilterValueRange::ArtworkNone => format!("not {base_sql}"),
        _ => return Err(invalid_filter_range_for_target(filter)),
    })
}

/// 指定されたフィルターに相当するSQL文のWHERE条件を取得(日付用)
/// # Arguments
/// - filter 対象のフィルタ
/// # Returns
/// 条件文(空文字列なら条件なし)
fn get_query_filter_where_date(filter: &Filter) -> Result<String> {
    let clm_name = target_to_clm_name(filter.target);

    Ok(match filter.range {
        //指定値と等しい
        FilterValueRange::DateEqual => {
            if !filter.str_value.is_empty() {
                format!("{} = {}", clm_name, escs(&filter.str_value))
            } else {
                format!("{clm_name} is null")
            }
        }
        //指定値と等しくない
        //※nullは含めない仕様(WalkBase1がそうなっていたので)
        FilterValueRange::DateNotEqual => {
            if !filter.str_value.is_empty() {
                format!("{} <> {}", clm_name, escs(&filter.str_value))
            } else {
                format!("{clm_name} is not null")
            }
        }
        //指定値以前
        FilterValueRange::DateBefore => {
            if !filter.str_value.is_empty() {
                format!("{} <= {}", clm_name, escs(&filter.str_value))
            } else {
                FALSE_QUERY.to_owned()
            }
        }
        //指定値以後
        FilterValueRange::DateAfter => {
            if !filter.str_value.is_empty() {
                format!("{} >= {}", clm_name, escs(&filter.str_value))
            } else {
                FALSE_QUERY.to_owned()
            }
        }
        //なし
        FilterValueRange::DateNone => format!("{clm_name} is null"),
        _ => return Err(invalid_filter_range_for_target(filter)),
    })
}

/// フィルタ対象列挙値を、songテーブルのカラム名に変換
fn target_to_clm_name(target: FilterTarget) -> &'static str {
    match target {
        FilterTarget::Title => "[title]",
        FilterTarget::Artist => "[artist]",
        FilterTarget::Album => "[album]",
        FilterTarget::Genre => "[genre]",
        FilterTarget::Albumartist => "[album_artist]",
        FilterTarget::Composer => "[composer]",
        FilterTarget::TrackNumber => "[track_number]",
        FilterTarget::TrackMax => "[track_max]",
        FilterTarget::DiscNumber => "[disc_number]",
        FilterTarget::DiscMax => "[disc_max]",
        FilterTarget::ReleaseDate => "[release_date]",
        FilterTarget::Rating => "[rating]",
        FilterTarget::Duration => "[duration]",
        FilterTarget::Memo => "[memo]",
        FilterTarget::MemoManage => "[memo_manage]",
        FilterTarget::EntryDate => "[entry_date]",
        FilterTarget::OriginalSong => "[original_song]",
        FilterTarget::SuggestTarget => "[suggest_target]",
        FilterTarget::FilterGroup | FilterTarget::Tags | FilterTarget::Artwork => unreachable!(),
    }
}

/// strValue1とstrValue2の数値の並び順を判別
/// # Returns
/// - .0: 小さい方の値
/// - .1: 大きい方の値
fn get_ordered_int(filter: &Filter) -> Result<(i32, i32)> {
    let val1 = i32::from_str(&filter.str_value)?;
    let val2 = i32::from_str(&filter.str_value2)?;
    if val1 <= val2 {
        Ok((val1, val2))
    } else {
        Ok((val2, val1))
    }
}

/// 必ずfalseを返す条件文
const FALSE_QUERY: &str = "0 = 1";

/// Error::InvalidFilterRangeForTargetのエラーを作成
fn invalid_filter_range_for_target(filter: &Filter) -> anyhow::Error {
    Error::InvalidFilterRangeForTarget {
        filter_id: filter.rowid,
        target: filter.target,
        range: filter.range,
    }
    .into()
}
