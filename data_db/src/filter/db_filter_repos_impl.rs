use super::{FilterDao, FilterRow};
use crate::{Error, error::FilterNoParentsDetectedItem};
use anyhow::Result;
use domain::{
    db_wrapper::TransactionWrapper,
    filter::{DbFilterRepository, Filter},
};
use std::rc::Rc;

/// DbFilterRepositoryの本実装
#[derive(new)]
pub struct DbFilterRepositoryImpl {
    filter_dao: Rc<dyn FilterDao>,
}

impl DbFilterRepository for DbFilterRepositoryImpl {
    /// フィルタをツリー構造で取得
    /// # Arguments
    /// - root_id:  最上位フィルタのID
    /// # Returns
    /// ツリー構造が作られたフィルタ
    fn get_filter_tree<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        root_id: i32,
    ) -> Result<Option<Filter>> {
        //まず該当rootIdのフィルタを全抽出
        let mut remain_pool = self.filter_dao.select_by_root_id(tx, root_id)?;

        if remain_pool.is_empty() {
            return Ok(None);
        }

        //最上位を取り出す
        let index = remain_pool
            .iter()
            .position(|f| f.root_id == root_id)
            .ok_or(Error::RootFilterNotFound { root_id })?;
        let root_row = remain_pool.remove(index);
        let mut root = Filter::from(root_row);

        let tuple = build_filter_children_recursive(root_id, remain_pool);
        root.children = tuple.0;

        if !tuple.1.is_empty() {
            return Err(Error::FilterNoParentsDetected(
                tuple
                    .1
                    .into_iter()
                    .map(|row| FilterNoParentsDetectedItem {
                        filter_id: row.rowid,
                        parent_id: row.parent_id,
                        root_id: row.root_id,
                    })
                    .collect(),
            )
            .into());
        }

        Ok(Some(root))
    }
}

/// 再帰的に子フィルタのツリーを構築
/// # Arguments
/// - parent_id: 構築対象のフィルタの親のID
/// - remain_pool: まだ親が見つかっていないフィルタ一覧
/// # Returns
/// - 0: `parent`引数を親に持つフィルタのリスト(子リスト構築済み)
/// - 1: まだ親が見つかってないフィルタ
fn build_filter_children_recursive(
    parent_id: i32,
    remain_pool: Vec<FilterRow>,
) -> (Vec<Filter>, Vec<FilterRow>) {
    //親プレイリストが対象のものと、それ以外を分ける
    let (targets, mut remain_pool): (Vec<FilterRow>, Vec<FilterRow>) = remain_pool
        .into_iter()
        .partition(|row| row.parent_id == Some(parent_id));

    let mut result_list = Vec::new();

    for target in targets {
        let mut filter = Filter::from(target);

        //再帰実行して子フィルタ一覧を取得
        let tuple = build_filter_children_recursive(filter.rowid, remain_pool);
        filter.children = tuple.0;
        remain_pool = tuple.1;

        result_list.push(filter);
    }

    (result_list, remain_pool)
}
