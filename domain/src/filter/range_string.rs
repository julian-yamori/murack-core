use serde::{Deserialize, Serialize};

use crate::db_utils::{escs, like_esc};

/// 文字列で絞り込み
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum StringFilterRange {
    /// 指定文字列と等しい
    Equal { value: String },

    /// 指定文字列と異なる
    NotEqual { value: String },

    /// 指定文字列を含む
    Contain { value: String },

    /// 指定文字列を含まない
    NotContain { value: String },

    /// 指定文字列から始まる
    Start { value: String },

    /// 指定文字列で終わる
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
