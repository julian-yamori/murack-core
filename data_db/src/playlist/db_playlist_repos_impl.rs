use anyhow::Result;
use async_trait::async_trait;
use domain::{
    db::DbTransaction,
    playlist::{DbPlaylistRepository, Playlist, PlaylistType},
};

use super::{PlaylistDao, PlaylistRow};
use crate::{Error, error::PlaylistNoParentsDetectedItem};

/// DbPlaylistRepositoryの本実装
#[derive(new)]
pub struct DbPlaylistRepositoryImpl<PD>
where
    PD: PlaylistDao + Sync + Send,
{
    playlist_dao: PD,
}

#[async_trait]
impl<PD> DbPlaylistRepository for DbPlaylistRepositoryImpl<PD>
where
    PD: PlaylistDao + Sync + Send,
{
    /// IDを指定してプレイリストを検索
    /// # Arguments
    /// id: playlist.rowid
    async fn get_playlist<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        id: i32,
    ) -> Result<Option<Playlist>> {
        let opt = self.playlist_dao.select_by_id(tx, id).await?;

        match opt {
            Some(row) => Ok(Some(row.try_into()?)),
            None => Ok(None),
        }
    }

    /// プレイリストのツリー構造を取得
    /// # Returns
    /// 最上位プレイリストのリスト
    async fn get_playlist_tree<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<Vec<Playlist>> {
        let remain_pool = self.playlist_dao.select_all_order_folder(tx).await?;

        let (root_list, remain_pool) = build_plist_children_recursive(None, remain_pool)?;

        if !remain_pool.is_empty() {
            return Err(Error::PlaylistNoParentsDetected(
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
    async fn reset_listuped_flag<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<()> {
        sqlx::query!(
            "UPDATE playlists SET listuped_flag = $1 WHERE playlist_type IN ($2::playlist_type, $3::playlist_type)",
            false,
            PlaylistType::Filter as PlaylistType,
            PlaylistType::Folder as PlaylistType
        )
        .execute(&mut **tx.get())
        .await?;

        Ok(())
    }

    /// 全プレイリストの、DAPに保存してからの変更フラグを設定
    /// # Arguments
    /// - is_changed: 変更されたか
    async fn set_dap_change_flag_all<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        is_changed: bool,
    ) -> Result<()> {
        sqlx::query!("UPDATE playlists SET dap_changed = $1", is_changed,)
            .execute(&mut **tx.get())
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
    parent: Option<&Playlist>,
    remain_pool: Vec<PlaylistRow>,
) -> anyhow::Result<(Vec<Playlist>, Vec<PlaylistRow>)> {
    //親プレイリストが対象のものと、それ以外を分ける
    let (targets, mut remain_pool): (Vec<PlaylistRow>, Vec<PlaylistRow>) = remain_pool
        .into_iter()
        .partition(|row| row.parent_id == parent.map(|p| p.rowid));

    let mut result_list = Vec::new();

    for target in targets {
        let mut plist = Playlist::try_from(target)?;

        //親プレイリスト名を親から繋げる
        if let Some(p) = parent {
            plist.parent_names = p.parent_names.clone();
            plist.parent_names.push(p.name.clone());
        }

        //再帰実行して子プレイリスト一覧を取得
        let tuple = build_plist_children_recursive(Some(&plist), remain_pool)?;
        plist.children = tuple.0;
        remain_pool = tuple.1;

        result_list.push(plist);
    }

    Ok((result_list, remain_pool))
}

#[cfg(test)]
mod tests {
    use domain::playlist::SortType;

    use super::super::MockPlaylistDao;
    use super::*;

    fn target() -> DbPlaylistRepositoryImpl<MockPlaylistDao> {
        DbPlaylistRepositoryImpl {
            playlist_dao: MockPlaylistDao::default(),
        }
    }
    fn checkpoint_all(target: &mut DbPlaylistRepositoryImpl<MockPlaylistDao>) {
        target.playlist_dao.inner.checkpoint();
    }

    /// get_playlist_treeテスト用のPlaylistRow作成
    fn tree_test_row(id: i32, name: &str, parent_id: Option<i32>) -> PlaylistRow {
        PlaylistRow {
            id,
            name: name.to_owned(),
            parent_id,

            playlist_type: PlaylistType::Normal,
            in_folder_order: 0,
            filter_json: None,
            sort_type: SortType::Artist,
            sort_desc: false,
            save_dap: true,
            listuped_flag: false,
            dap_changed: true,
        }
    }
    /// get_playlist_treeテスト用のPlaylist作成
    fn tree_test_model(
        rowid: i32,
        name: &str,
        parent_id: Option<i32>,
        parent_names: Vec<&str>,
        children: Vec<Playlist>,
    ) -> Playlist {
        Playlist {
            rowid,
            name: name.to_owned(),
            parent_id,
            parent_names: parent_names.into_iter().map(|s| (*s).to_owned()).collect(),
            children,

            playlist_type: PlaylistType::Normal,
            in_folder_order: 0,
            filter: None,
            sort_type: SortType::Artist,
            sort_desc: false,
            save_dap: true,
            listuped_flag: false,
            dap_changed: true,
        }
    }

    #[tokio::test]
    async fn test_get_playlist_tree_empty() -> anyhow::Result<()> {
        let mut target = target();
        target
            .playlist_dao
            .inner
            .expect_select_all_order_folder()
            .returning(|| Ok(vec![]));

        let mut tx = DbTransaction::Dummy;

        let result = target.get_playlist_tree(&mut tx).await?;
        assert_eq!(result, vec![]);

        checkpoint_all(&mut target);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_playlist_tree_flat() -> anyhow::Result<()> {
        let mut target = target();
        target
            .playlist_dao
            .inner
            .expect_select_all_order_folder()
            .returning(|| {
                Ok(vec![
                    tree_test_row(3, "one", None),
                    tree_test_row(5, "two", None),
                    tree_test_row(2, "three", None),
                ])
            });

        let mut tx = DbTransaction::Dummy;

        let result = target.get_playlist_tree(&mut tx).await?;
        pretty_assertions::assert_eq!(
            result,
            vec![
                tree_test_model(3, "one", None, vec![], vec![]),
                tree_test_model(5, "two", None, vec![], vec![]),
                tree_test_model(2, "three", None, vec![], vec![]),
            ]
        );

        checkpoint_all(&mut target);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_playlist_tree_treed() -> anyhow::Result<()> {
        let mut target = target();
        target
            .playlist_dao
            .inner
            .expect_select_all_order_folder()
            .returning(|| {
                Ok(vec![
                    tree_test_row(3, "1-1", Some(5)),
                    tree_test_row(5, "root1", None),
                    tree_test_row(9, "1-2-1", Some(2)),
                    tree_test_row(2, "1-2", Some(5)),
                    tree_test_row(35, "root2", None),
                    tree_test_row(75, "2-1", Some(35)),
                    tree_test_row(98, "1-2-2", Some(2)),
                    tree_test_row(1, "1-3", Some(5)),
                ])
            });

        let mut tx = DbTransaction::Dummy;

        let result = target.get_playlist_tree(&mut tx).await?;
        pretty_assertions::assert_eq!(
            result,
            vec![
                tree_test_model(
                    5,
                    "root1",
                    None,
                    vec![],
                    vec![
                        tree_test_model(3, "1-1", Some(5), vec!["root1"], vec![]),
                        tree_test_model(
                            2,
                            "1-2",
                            Some(5),
                            vec!["root1"],
                            vec![
                                tree_test_model(9, "1-2-1", Some(2), vec!["root1", "1-2"], vec![]),
                                tree_test_model(98, "1-2-2", Some(2), vec!["root1", "1-2"], vec![]),
                            ]
                        ),
                        tree_test_model(1, "1-3", Some(5), vec!["root1"], vec![]),
                    ]
                ),
                tree_test_model(
                    35,
                    "root2",
                    None,
                    vec![],
                    vec![tree_test_model(75, "2-1", Some(35), vec!["root2"], vec![]),]
                ),
            ]
        );

        checkpoint_all(&mut target);
        Ok(())
    }
}
