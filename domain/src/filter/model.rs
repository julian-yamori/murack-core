use super::{FilterTarget, FilterValueRange};

/// フィルター データ
#[derive(Debug, PartialEq, Clone)]
pub struct Filter {
    /// PK
    pub rowid: i32,

    /// フィルタ種類
    pub target: FilterTarget,

    /// フィルタリング比較値1
    pub str_value: String,

    /// フィルタリング比較値2
    ///
    /// 2値で範囲を指定するための値
    pub str_value2: String,

    /// 値の範囲指定
    pub range: FilterValueRange,

    /// 子フィルター
    pub children: Vec<Filter>,
}
