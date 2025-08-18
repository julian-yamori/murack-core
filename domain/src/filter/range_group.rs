use serde::{Deserialize, Serialize};

use crate::filter::FilterTarget;

/// `FilterTarget::FilterGroup` を、SQL の WHERE で使用する条件式に変換
///
/// フィルタ条件が無い場合は None (空の Group しか無い場合)
pub fn group_where_expression(op: &GroupOperand, children: &[FilterTarget]) -> Option<String> {
    //各フィルタのクエリを連結
    let expressions = children
        .iter()
        .filter_map(FilterTarget::where_expression)
        .collect::<Vec<String>>();

    if expressions.is_empty() {
        return None;
    }

    let ope = match op {
        GroupOperand::And => " and ",
        GroupOperand::Or => " or ",
    };

    //クエリ文字列は()で囲む
    Some(format!("({})", expressions.join(ope)))
}

/// 集合フィルタの条件指定方法
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupOperand {
    /// 全てを満たす
    And,

    /// いずれかを満たす
    Or,
}
