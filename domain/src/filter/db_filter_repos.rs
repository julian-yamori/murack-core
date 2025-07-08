use super::Filter;
use crate::db_wrapper::TransactionWrapper;
use anyhow::Result;
use mockall::automock;

/// フィルタ関係のDBリポジトリ
#[automock]
pub trait DbFilterRepository {
    /// フィルタをツリー構造で取得
    /// # Arguments
    /// - root_id:  最上位フィルタのID
    /// # Returns
    /// ツリー構造が作られたフィルタ
    fn get_filter_tree<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        root_id: i32,
    ) -> Result<Option<Filter>>;
}
