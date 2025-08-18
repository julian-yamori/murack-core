use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use test_case::test_case;

    #[test_case(15, 28, 15, 28 ; "normal")]
    #[test_case(28, 15, 15, 28 ; "inversed")]
    #[test_case(6, 113, 6, 113 ; "digit_dif")]
    #[test_case(113, 6, 6, 113 ; "digit_dif_inversed")]
    fn test_get_ordered_int(value1: i32, value2: i32, expect_1: i32, expect_2: i32) {
        let result = super::get_ordered_int(value1, value2);
        assert_eq!(result.0, expect_1);
        assert_eq!(result.1, expect_2);
    }
}
