use super::{PlaylistDao, PlaylistRow};
use crate::{converts::DbPlaylistType, error::PlaylistNoParentsDetectedItem, sql_func, Error};
use anyhow::Result;
use domain::{
    db_wrapper::TransactionWrapper,
    playlist::{DbPlaylistRepository, Playlist, PlaylistType},
};
use rusqlite::params;
use std::rc::Rc;

/// DbPlaylistRepositoryの本実装
#[derive(new)]
pub struct DbPlaylistRepositoryImpl {
    playlist_dao: Rc<dyn PlaylistDao>,
}

impl DbPlaylistRepository for DbPlaylistRepositoryImpl {
    /// IDを指定してプレイリストを検索
    /// # Arguments
    /// id: playlist.rowid
    fn get_playlist<'c>(&self, tx: &TransactionWrapper<'c>, id: i32) -> Result<Option<Playlist>> {
        Ok(self.playlist_dao.select_by_id(tx, id)?.map(Playlist::from))
    }

    /// プレイリストのツリー構造を取得
    /// # Returns
    /// 最上位プレイリストのリスト
    fn get_playlist_tree<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<Vec<Playlist>> {
        let remain_pool = self.playlist_dao.select_all_order_folder(tx)?;

        let (root_list, remain_pool) = build_plist_children_recursive(None, remain_pool);

        if !remain_pool.is_empty() {
            return Err(Error::PlaylistNoParentsDetected(
                remain_pool
                    .into_iter()
                    .map(|row| PlaylistNoParentsDetectedItem {
                        playlist_id: row.rowid,
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
    fn reset_listuped_flag(&self, tx: &TransactionWrapper) -> Result<()> {
        sql_func::execute(
            tx,
            "update [playlist] set [listuped_flag] = ? where [type] in (?,?)",
            params![
                false,
                DbPlaylistType::from(PlaylistType::Filter),
                DbPlaylistType::from(PlaylistType::Folder)
            ],
        )
    }

    /// 全プレイリストの、DAPに保存してからの変更フラグを設定
    /// # Arguments
    /// - is_changed: 変更されたか
    fn set_dap_change_flag_all<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        is_changed: bool,
    ) -> Result<()> {
        sql_func::execute(
            tx,
            "update [playlist] set [dap_changed] = ?",
            params![is_changed],
        )
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
) -> (Vec<Playlist>, Vec<PlaylistRow>) {
    //親プレイリストが対象のものと、それ以外を分ける
    let (targets, mut remain_pool): (Vec<PlaylistRow>, Vec<PlaylistRow>) = remain_pool
        .into_iter()
        .partition(|row| row.parent_id == parent.map(|p| p.rowid));

    let mut result_list = Vec::new();

    for target in targets {
        let mut plist = Playlist::from(target);

        //親プレイリスト名を親から繋げる
        if let Some(p) = parent {
            plist.parent_names = p.parent_names.clone();
            plist.parent_names.push(p.name.clone());
        }

        //再帰実行して子プレイリスト一覧を取得
        let tuple = build_plist_children_recursive(Some(&plist), remain_pool);
        plist.children = tuple.0;
        remain_pool = tuple.1;

        result_list.push(plist);
    }

    (result_list, remain_pool)
}

#[cfg(test)]
mod tests {
    use super::super::MockPlaylistDao;
    use super::*;
    use domain::{db_wrapper::ConnectionFactory, mocks, playlist::SortType};
    use paste::paste;

    mocks! {
        DbPlaylistRepositoryImpl,
        [PlaylistDao]
    }

    /// get_playlist_treeテスト用のPlaylistRow作成
    fn tree_test_row(rowid: i32, name: &str, parent_id: Option<i32>) -> PlaylistRow {
        PlaylistRow {
            rowid,
            name: name.to_owned(),
            parent_id,

            playlist_type: PlaylistType::Normal.into(),
            in_folder_order: 0,
            filter_root_id: None,
            sort_type: SortType::Artist.into(),
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
            filter_root_id: None,
            sort_type: SortType::Artist,
            sort_desc: false,
            save_dap: true,
            listuped_flag: false,
            dap_changed: true,
        }
    }

    #[test]
    fn test_get_playlist_tree_empty() {
        let mut mocks = Mocks::new();
        mocks.playlist_dao(|m| {
            m.expect_select_all_order_folder().returning(|_| Ok(vec![]));
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let result = t.get_playlist_tree(&tx).unwrap();
            assert_eq!(result, vec![]);
        });
    }

    #[test]
    fn test_get_playlist_tree_flat() {
        let mut mocks = Mocks::new();
        mocks.playlist_dao(|m| {
            m.expect_select_all_order_folder().returning(|_| {
                Ok(vec![
                    tree_test_row(3, "one", None),
                    tree_test_row(5, "two", None),
                    tree_test_row(2, "three", None),
                ])
            });
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let result = t.get_playlist_tree(&tx).unwrap();
            pretty_assertions::assert_eq!(
                result,
                vec![
                    tree_test_model(3, "one", None, vec![], vec![]),
                    tree_test_model(5, "two", None, vec![], vec![]),
                    tree_test_model(2, "three", None, vec![], vec![]),
                ]
            );
        });
    }

    #[test]
    fn test_get_playlist_tree_treed() {
        let mut mocks = Mocks::new();
        mocks.playlist_dao(|m| {
            m.expect_select_all_order_folder().returning(|_| {
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
        });

        let mut db = ConnectionFactory::Dummy.open().unwrap();
        let tx = db.transaction().unwrap();

        mocks.run_target(|t| {
            let result = t.get_playlist_tree(&tx).unwrap();
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
                                    tree_test_model(
                                        9,
                                        "1-2-1",
                                        Some(2),
                                        vec!["root1", "1-2"],
                                        vec![]
                                    ),
                                    tree_test_model(
                                        98,
                                        "1-2-2",
                                        Some(2),
                                        vec!["root1", "1-2"],
                                        vec![]
                                    ),
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
        });
    }
}
