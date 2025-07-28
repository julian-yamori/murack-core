use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgTransaction;

use crate::{
    NonEmptyString,
    playlist::{
        DbPlaylistRepository, Playlist, PlaylistRow, PlaylistTree, PlaylistType, SortType,
        playlist_error::{PlaylistError, PlaylistNoParentsDetectedItem},
    },
};

/// DbPlaylistRepositoryの本実装
#[derive(new)]
pub struct DbPlaylistRepositoryImpl {}

#[async_trait]
impl DbPlaylistRepository for DbPlaylistRepositoryImpl {
    /// IDを指定してプレイリストを検索
    /// # Arguments
    /// id: playlist.rowid
    async fn get_playlist<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        id: i32,
    ) -> Result<Option<Playlist>> {
        let opt = sqlx::query_as!(
            PlaylistRow,
            r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name AS "name: NonEmptyString", parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists WHERE id = $1"#,
            id
        )
        .fetch_optional(&mut **tx)
        .await?;

        match opt {
            Some(row) => Ok(Some(row.try_into()?)),
            None => Ok(None),
        }
    }

    /// プレイリストのツリー構造を取得
    /// # Returns
    /// 最上位プレイリストのリスト
    async fn get_playlist_tree<'c>(&self, tx: &mut PgTransaction<'c>) -> Result<Vec<PlaylistTree>> {
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

    /// 全フィルタプレイリスト・フォルダプレイリストの、リストアップ済みフラグを解除する。
    async fn reset_listuped_flag<'c>(&self, tx: &mut PgTransaction<'c>) -> Result<()> {
        sqlx::query!(
            "UPDATE playlists SET listuped_flag = $1 WHERE playlist_type IN ($2::playlist_type, $3::playlist_type)",
            false,
            PlaylistType::Filter as PlaylistType,
            PlaylistType::Folder as PlaylistType
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 全プレイリストの、DAPに保存してからの変更フラグを設定
    /// # Arguments
    /// - is_changed: 変更されたか
    async fn set_dap_change_flag_all<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        is_changed: bool,
    ) -> Result<()> {
        sqlx::query!("UPDATE playlists SET dap_changed = $1", is_changed,)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
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
        .partition(|row| row.parent_id == parent.map(|p| p.playlist.rowid));

    let mut result_list = Vec::new();

    for target in targets {
        let mut current_tree = PlaylistTree {
            playlist: Playlist::try_from(target)?,
            children: vec![],
            parent_names: vec![],
        };

        //親プレイリスト名を親から繋げる
        if let Some(parent_tree) = parent {
            current_tree.parent_names = parent_tree.parent_names.clone();
            current_tree
                .parent_names
                .push(parent_tree.playlist.name.clone());
        }

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
