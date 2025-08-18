use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::db_utils::escs;

/// 日付で絞り込み
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum DateFilterRange {
    /// 日付：指定値と等しい
    Equal { value: NaiveDate },

    /// 日付：指定値と異なる
    NotEqual { value: NaiveDate },

    /// 日付：指定値以前
    Before { value: NaiveDate },

    /// 日付：指定値以後
    After { value: NaiveDate },

    /// 日付：ない
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
