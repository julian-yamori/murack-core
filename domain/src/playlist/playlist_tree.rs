#[cfg(test)]
mod tests;

use std::fmt::Debug;

use crate::playlist::playlist_error::{PlaylistError, PlaylistNoParentsDetectedItem};

/// プレイリストのツリー構造
pub struct PlaylistTree<V: PlaylistTreeValue> {
    pub value: V,
    pub children: Vec<PlaylistTree<V>>,
}

impl<V: PlaylistTreeValue> PlaylistTree<V> {
    /// DB 全体のプレイリストの Vec を、ツリー構造に変換
    pub fn from_all_playlists(remain_pool: Vec<V>) -> Result<Vec<PlaylistTree<V>>, PlaylistError> {
        let RecursiveOk {
            children: root_list,
            remain_pool,
        } = build_node_children(None, remain_pool)?;

        if !remain_pool.is_empty() {
            return Err(PlaylistError::PlaylistNoParentsDetected(
                remain_pool
                    .into_iter()
                    .map(|row| PlaylistNoParentsDetectedItem {
                        playlist_id: row.id(),
                        parent_id: row.parent_id(),
                    })
                    .collect(),
            ));
        }

        Ok(root_list)
    }
}

/// 再帰的に子プレイリストのツリーを構築
/// # Arguments
/// - parent_id: 構築対象のプレイリストの親の ID (None なら Root)
/// - remain_pool: まだ親が見つかっていないプレイリスト一覧
fn build_node_children<V: PlaylistTreeValue>(
    parent_id: Option<i32>,
    remain_pool: Vec<V>,
) -> Result<RecursiveOk<V>, PlaylistError> {
    //親プレイリストが対象のものと、それ以外を分ける
    let (mut targets, mut remain_pool): (Vec<V>, Vec<V>) = remain_pool
        .into_iter()
        .partition(|value| value.parent_id() == parent_id);

    let mut result_list = Vec::new();

    targets.sort_by_key(|a| a.in_folder_order());

    for value in targets {
        //再帰実行して子プレイリスト一覧を取得
        let result = build_node_children(Some(value.id()), remain_pool)?;

        let current_tree = PlaylistTree {
            value,
            children: result.children,
        };
        remain_pool = result.remain_pool;

        result_list.push(current_tree);
    }

    Ok(RecursiveOk {
        children: result_list,
        remain_pool,
    })
}

/// build_node_children の Ok 時の戻り値
struct RecursiveOk<V: PlaylistTreeValue> {
    /// 親 ID が `parent_id` 引数と一致するプレイリストのリスト
    children: Vec<PlaylistTree<V>>,

    /// まだ親が見つかってないプレイリスト
    remain_pool: Vec<V>,
}

impl<V> Debug for PlaylistTree<V>
where
    V: PlaylistTreeValue + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlaylistTree")
            .field("value", &self.value)
            .field("children", &self.children)
            .finish()
    }
}

impl<V> PartialEq for PlaylistTree<V>
where
    V: PlaylistTreeValue + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.children == other.children
    }
}

impl<V> Eq for PlaylistTree<V> where V: PlaylistTreeValue + Eq {}

impl<V> Clone for PlaylistTree<V>
where
    V: PlaylistTreeValue + Clone,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            children: self.children.clone(),
        }
    }
}

impl<V> Default for PlaylistTree<V>
where
    V: PlaylistTreeValue + Default,
{
    fn default() -> Self {
        Self {
            value: Default::default(),
            children: Default::default(),
        }
    }
}

/// PlaylistTree の値となるプレイリストモデル
pub trait PlaylistTreeValue {
    fn id(&self) -> i32;

    fn parent_id(&self) -> Option<i32>;

    fn in_folder_order(&self) -> u32;
}
