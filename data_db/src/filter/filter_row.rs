use crate::converts::{DbFilterTarget, DbFilterValueRange};
use domain::filter::Filter;

/// filterテーブルのレコード
#[derive(Debug, PartialEq, Clone)]
pub struct FilterRow {
    /// PK
    pub rowid: i32,

    /// 親FilterのId
    ///
    /// 親になれるフィルターはEFilterTarget.FilterGroupのみ。
    /// 最上位ならnull。
    pub parent_id: Option<i32>,

    /// 親フィルタ内での並び順
    ///
    /// 最上位は0固定
    pub in_parent_order: i32,

    /// このフィルターの最上位フィルターのID
    pub root_id: i32,

    /// フィルタ種類
    pub target: DbFilterTarget,

    /// フィルタリング比較値1
    pub str_value: String,

    /// フィルタリング比較値2
    ///
    /// 2値で範囲を指定するための値
    pub str_value2: String,

    /// 値の範囲指定
    pub range: DbFilterValueRange,
}

impl From<FilterRow> for Filter {
    fn from(f: FilterRow) -> Self {
        Self {
            rowid: f.rowid,
            target: f.target.into(),
            str_value: f.str_value,
            str_value2: f.str_value2,
            range: f.range.into(),
            children: vec![],
        }
    }
}
