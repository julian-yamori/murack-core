use serde::{Deserialize, Serialize};

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
