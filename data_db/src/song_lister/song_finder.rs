use super::{SongListerFilter, esc::esci};
use crate::{
    Error,
    converts::DbLibSongPath,
    playlist::{PlaylistDao, PlaylistSongDao},
    sql_func,
};
use anyhow::Result;
use domain::{
    dap::SongFinder,
    db_wrapper::TransactionWrapper,
    filter::DbFilterRepository,
    path::LibSongPath,
    playlist::{Playlist, PlaylistType, SortType},
};
use rusqlite::params;
use std::{
    collections::{BTreeSet, HashSet},
    rc::Rc,
};

/// SongFinderの本実装
#[derive(new)]
pub struct SongFinderImpl {
    playlist_dao: Rc<dyn PlaylistDao>,
    playlist_song_dao: Rc<dyn PlaylistSongDao>,
    db_filter_repository: Rc<dyn DbFilterRepository>,
    song_lister_filter: Rc<dyn SongListerFilter>,
}

impl SongFinder for SongFinderImpl {
    /// プレイリストに含まれる曲のパスリストを取得
    /// # Arguments
    /// - plist 取得対象のプレイリスト情報(※childrenは不要)
    fn get_song_path_list<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        plist: &Playlist,
    ) -> Result<Vec<LibSongPath>> {
        //対象プレイリストのクエリ(from,join,where句)を取得
        let fjw_query = self.get_query_by_playlist(tx, plist)?;

        //取得するカラム
        let mut clms_query = "[song].[path]".to_owned();

        //プレイリスト順なら、取得カラムを一つ追加
        if plist.sort_type == SortType::Playlist {
            clms_query = format!(
                "{clms_query}, [playlist_song].[order] as [{PLIST_SONG_IDX_COLUMN}]"
            );
        }

        //select句とorder byを結合
        let query = format!(
            "select {}{}{}",
            clms_query,
            fjw_query,
            get_order_query(plist.sort_type, plist.sort_desc)
        );

        sql_func::select_list(tx, &query, [], |row| {
            let p: DbLibSongPath = row.get(0)?;
            Ok(p.into())
        })
    }
}

/// playlist_song.orderカラムに付ける別名
const PLIST_SONG_IDX_COLUMN: &str = "playlist_index";

impl SongFinderImpl {
    /// プレイリストに含まれる曲を検索するクエリを作成
    /// # Arguments
    /// - plist: 対象プレイリスト情報
    /// # Result
    /// from,join,where句のクエリ
    fn get_query_by_playlist(&self, tx: &TransactionWrapper, plist: &Playlist) -> Result<String> {
        //リストアップされていなければ、まずリストアップする
        if !plist.listuped_flag {
            self.listup_songs(tx, plist)?;
        }
        //プレイリストに含まれる曲の検索クエリを返す

        Ok(format!(
            " from [playlist_song] join [song] on [playlist_song].[song_id] = [song].[rowid] where [playlist_song].[playlist_id] = {}",
            esci(Some(plist.rowid))
        ))
    }

    /// プレイリストの曲をリストアップし、playlist_songテーブルを更新する
    /// # Arguments
    /// - plist: 対象プレイリスト情報
    fn listup_songs(&self, tx: &TransactionWrapper, plist: &Playlist) -> Result<()> {
        //通常プレイリストなら、リストアップ済みフラグを立てるのみ
        if plist.playlist_type != PlaylistType::Normal {
            //元々保存されていた曲リストを取得
            let old_id_list: BTreeSet<_> = self
                .playlist_song_dao
                .select_song_id_by_playlist_id(tx, plist.rowid)?
                .into_iter()
                .collect();

            let new_id_list = match plist.playlist_type {
                PlaylistType::Filter => self.search_plist_songs_filter(tx, plist)?,
                PlaylistType::Folder => self.search_plist_songs_folder(tx, plist)?,
                _ => unreachable!(),
            };

            //PlaylistSongテーブルを更新
            self.playlist_song_dao
                .delete_by_playlist_id(tx, plist.rowid)?;
            for (idx, song_id) in new_id_list.iter().enumerate() {
                self.playlist_song_dao
                    .insert(tx, plist.rowid, *song_id, idx as i32)?;
            }

            //古いリストから変更があったか確認
            let mut changed = false;
            if old_id_list.len() != new_id_list.len() {
                changed = true;
            } else {
                for id in new_id_list {
                    if !old_id_list.contains(&id) {
                        changed = true;
                        break;
                    }
                }
            }

            //変更があれば、DAP変更フラグを立てる
            if changed {
                sql_func::execute(
                    tx,
                    "update [playlist] set [dap_changed] = 1 where [rowid] = ?",
                    params![plist.rowid],
                )?;
            }
        }

        //リストアップ済みに更新
        sql_func::execute(
            tx,
            "update [playlist] set [listuped_flag] = ? where [rowid] = ?",
            params![true, plist.rowid],
        )
    }

    /// プレイリストの設定に基づき、曲リストを取得：フォルダプレイリスト
    /// # Arguments
    /// - plist: 対象プレイリスト情報
    fn search_plist_songs_folder(
        &self,
        tx: &TransactionWrapper,
        plist: &Playlist,
    ) -> Result<Vec<i32>> {
        //直下の子のプレイリストを取得
        let children = self
            .playlist_dao
            .get_child_playlists(tx, Some(plist.rowid))?
            .into_iter()
            .map(Playlist::from);

        //子プレイリストの曲IDを追加していくSet
        let mut add_song_ids = HashSet::<i32>::new();

        for child in children {
            //子プレイリストの曲リストを取得
            let child_query = format!(
                "select [song].[rowid] {}",
                self.get_query_by_playlist(tx, &child)?
            );
            let child_songs: Vec<i32> =
                sql_func::select_list(tx, &child_query, [], |row| row.get(0))?;

            //Setに追加
            for song_id in child_songs {
                add_song_ids.insert(song_id);
            }
        }

        Ok(add_song_ids.into_iter().collect())
    }

    /// プレイリストの設定に基づき、曲リストを取得：フィルタプレイリスト
    /// # Arguments
    /// - plist: 対象プレイリスト情報
    fn search_plist_songs_filter(
        &self,
        tx: &TransactionWrapper,
        plist: &Playlist,
    ) -> Result<Vec<i32>> {
        let filter_id = plist
            .filter_root_id
            .ok_or(Error::FilterPlaylistFilterIdNone {
                plist_id: plist.rowid,
            })?;

        //該当フィルタを取得
        let filter = self
            .db_filter_repository
            .get_filter_tree(tx, filter_id)?
            .ok_or(Error::RootFilterNotFound { root_id: filter_id })?;

        self.song_lister_filter.list_song_id(tx, &filter)
    }
}

/// ORDER BYクエリを取得
/// # Arguments
/// - sort_type ソート対象
/// - is_desc ソートが降順か
/// # Return
/// order by句
fn get_order_query(sort_type: SortType, is_desc: bool) -> String {
    let order = get_sort_column_query(sort_type);

    //降順ならASC → DESC
    if is_desc {
        format!(" order by {}", order.replace("asc", "desc"))
    } else {
        format!(" order by {order}")
    }
}

/// カラムのソート順のクエリを取得
/// # Arguments
/// - sort_type: ソート対象
/// # Returns
/// order byに繋がる文字列。全ての列にasc付き
fn get_sort_column_query(sort_type: SortType) -> String {
    match sort_type {
	SortType::SongName => "[title_order] asc, [song].[rowid] asc".to_owned(),
	SortType::Artist => "[artist_order] asc, [album_order] asc, [disc_number] asc, [track_number] asc, [title_order] asc, [song].[rowid] asc".to_owned(),
	SortType::Album => "[album_order] asc, [artist_order] asc, [disc_number] asc, [track_number] asc, [title_order] asc, [song].[rowid] asc".to_owned(),
	SortType::Genre => "[genre] asc, [artist_order] asc, [album_order] asc, [disc_number] asc, [track_number] asc, [title_order] asc, [song].[rowid] asc".to_owned(),
	SortType::Playlist => format!("[{PLIST_SONG_IDX_COLUMN}] asc"),
	SortType::Composer => "[composer_order] asc, [artist_order] asc, [album_order] asc, [disc_number] asc, [track_number] asc, [title_order] asc, [song].[rowid] asc".to_owned(),
	SortType::Duration => "[duration] asc, [title_order] asc, [song].[rowid] asc".to_owned(),
	SortType::TrackIndex => "[track_number] asc, [artist_order] asc, [album_order] asc, [disc_number] asc, [title_order] asc, [song].[rowid] asc".to_owned(),
	SortType::DiscIndex => "[disc_number] asc, [artist_order] asc, [album_order] asc, [track_number] asc, [title_order] asc, [song].[rowid] asc".to_owned(),
	SortType::ReleaseDate => "[release_date] asc, [artist_order] asc, [album_order] asc, [disc_number] asc, [track_number] asc, [title_order] asc, [song].[rowid] asc".to_owned(),
	SortType::Rating => "[rating] asc, [artist_order] asc, [album_order] asc, [disc_number] asc, [track_number] asc, [title_order] asc, [song].[rowid] asc".to_owned(),
	SortType::EntryDate => "[entry_date] asc, [path] asc".to_owned(),
	SortType::Path => "[path] asc".to_owned(),
	}
}
