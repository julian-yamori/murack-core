use anyhow::Result;
use sqlx::PgTransaction;

use crate::{
    NonEmptyString,
    playlist::{
        Playlist, PlaylistRow, PlaylistTree, PlaylistType, SortType,
        playlist_error::{PlaylistError, PlaylistNoParentsDetectedItem},
    },
};

/// プレイリストのツリー構造を取得
/// # Returns
/// 最上位プレイリストのリスト
pub async fn get_playlist_tree<'c>(tx: &mut PgTransaction<'c>) -> Result<Vec<PlaylistTree>> {
    let remain_pool = sqlx::query_as!(
            PlaylistRow,
            r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name AS "name: NonEmptyString", parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists ORDER BY in_folder_order"#
        )
        .fetch_all(&mut **tx)
        .await?;

    let (root_list, remain_pool) = build_plist_children_recursive(None, remain_pool)?;

    if !remain_pool.is_empty() {
        return Err(PlaylistError::PlaylistNoParentsDetected(
            remain_pool
                .into_iter()
                .map(|row| PlaylistNoParentsDetectedItem {
                    playlist_id: row.id,
                    name: row.name,
                    parent_id: row.parent_id,
                })
                .collect(),
        )
        .into());
    }

    Ok(root_list)
}

/// 再帰的に子プレイリストのツリーを構築
/// # Arguments
/// - parent: 構築対象のプレイリストの親(NoneならRoot)
/// - remain_pool: まだ親が見つかっていないプレイリスト一覧
/// # Returns
/// - 0: `parent`引数を親に持つプレイリストのリスト(子リスト構築済み)
/// - 1: まだ親が見つかってないプレイリスト
fn build_plist_children_recursive(
    parent: Option<&PlaylistTree>,
    remain_pool: Vec<PlaylistRow>,
) -> anyhow::Result<(Vec<PlaylistTree>, Vec<PlaylistRow>)> {
    //親プレイリストが対象のものと、それ以外を分ける
    let (targets, mut remain_pool): (Vec<PlaylistRow>, Vec<PlaylistRow>) = remain_pool
        .into_iter()
        .partition(|row| row.parent_id == parent.map(|p| p.playlist.id));

    let mut result_list = Vec::new();

    for target in targets {
        let mut current_tree = PlaylistTree {
            playlist: Playlist::try_from(target)?,
            children: vec![],
        };

        //再帰実行して子プレイリスト一覧を取得
        let tuple = build_plist_children_recursive(Some(&current_tree), remain_pool)?;
        current_tree.children = tuple.0;
        remain_pool = tuple.1;

        result_list.push(current_tree);
    }

    Ok((result_list, remain_pool))
}

#[cfg(test)]
mod tests;
