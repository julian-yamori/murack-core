use std::collections::{BTreeSet, HashSet};

use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use murack_core_domain::{
    NonEmptyString,
    dap::TrackFinder,
    path::LibTrackPath,
    playlist::{Playlist, PlaylistType, SortType},
};
use sqlx::PgTransaction;
use sqlx::{Row, postgres::PgRow};

use super::{TrackListerFilter, esc::esci};
use crate::{
    Error,
    playlist::{PlaylistRow, playlist_sqls},
};

/// TrackFinderの本実装
#[derive(new)]
pub struct TrackFinderImpl<SLF>
where
    SLF: TrackListerFilter + Sync + Send,
{
    track_lister_filter: SLF,
}

#[async_trait]
impl<SLF> TrackFinder for TrackFinderImpl<SLF>
where
    SLF: TrackListerFilter + Sync + Send,
{
    /// プレイリストに含まれる曲のパスリストを取得
    /// # Arguments
    /// - plist 取得対象のプレイリスト情報(※childrenは不要)
    async fn get_track_path_list<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        plist: &Playlist,
    ) -> Result<Vec<LibTrackPath>> {
        //対象プレイリストのクエリ(from,join,where句)を取得
        let fjw_query = self.get_query_by_playlist(tx, plist).await?;

        //取得するカラム
        let mut clms_query = "tracks.path".to_owned();

        //プレイリスト順なら、取得カラムを一つ追加
        if plist.sort_type == SortType::Playlist {
            clms_query =
                format!("{clms_query}, playlist_tracks.order_index AS {PLIST_TRACK_IDX_COLUMN}");
        }

        //select句とorder byを結合
        let query = format!(
            "SELECT {}{}{}",
            clms_query,
            fjw_query,
            get_order_query(plist.sort_type, plist.sort_desc)
        );

        let list: Vec<LibTrackPath> = sqlx::query(&query)
            .map(|row: PgRow| row.get::<LibTrackPath, _>(0))
            .fetch_all(&mut **tx)
            .await?;

        Ok(list)
    }
}

/// playlist_track.orderカラムに付ける別名
const PLIST_TRACK_IDX_COLUMN: &str = "playlist_index";

impl<SLF> TrackFinderImpl<SLF>
where
    SLF: TrackListerFilter + Sync + Send,
{
    /// プレイリストに含まれる曲を検索するクエリを作成
    /// # Arguments
    /// - plist: 対象プレイリスト情報
    /// # Result
    /// from,join,where句のクエリ
    async fn get_query_by_playlist<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        plist: &Playlist,
    ) -> Result<String> {
        //リストアップされていなければ、まずリストアップする
        if !plist.listuped_flag {
            self.listup_tracks(tx, plist).await?;
        }
        //プレイリストに含まれる曲の検索クエリを返す

        Ok(format!(
            " FROM playlist_trakcs JOIN tracks ON playlist_tracks.track_id = tracks.id WHERE playlist_tracks.playlist_id = {}",
            esci(Some(plist.rowid))
        ))
    }

    /// プレイリストの曲をリストアップし、playlist_trackテーブルを更新する
    /// # Arguments
    /// - plist: 対象プレイリスト情報
    async fn listup_tracks<'c>(&self, tx: &mut PgTransaction<'c>, plist: &Playlist) -> Result<()> {
        //通常プレイリストなら、リストアップ済みフラグを立てるのみ
        if plist.playlist_type != PlaylistType::Normal {
            //元々保存されていた曲リストを取得
            let old_id_list: BTreeSet<_> =
                playlist_sqls::select_track_id_by_playlist_id(tx, plist.rowid)
                    .await?
                    .into_iter()
                    .collect();

            let new_id_list = match plist.playlist_type {
                PlaylistType::Filter => self.search_plist_tracks_filter(tx, plist).await?,
                PlaylistType::Folder => self.search_plist_tracks_folder(tx, plist).await?,
                _ => unreachable!(),
            };

            //PlaylistTrackテーブルを更新
            playlist_sqls::delete_by_playlist_id(tx, plist.rowid).await?;
            for (idx, track_id) in new_id_list.iter().enumerate() {
                playlist_sqls::insert_playlist_track(tx, plist.rowid, *track_id, idx as i32)
                    .await?;
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
                sqlx::query!(
                    "UPDATE playlists SET dap_changed = true WHERE id = $1",
                    plist.rowid,
                )
                .execute(&mut **tx)
                .await?;
            }
        }

        //リストアップ済みに更新
        sqlx::query!(
            "UPDATE playlists SET listuped_flag = $1 WHERE id = $2",
            true,
            plist.rowid,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// プレイリストの設定に基づき、曲リストを取得：フォルダプレイリスト
    /// # Arguments
    /// - plist: 対象プレイリスト情報
    #[async_recursion]
    async fn search_plist_tracks_folder<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        plist: &Playlist,
    ) -> Result<Vec<i32>> {
        let children = sqlx::query_as!(
            PlaylistRow,
            r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name AS "name: NonEmptyString", parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists WHERE parent_id IS NOT DISTINCT FROM $1 ORDER BY in_folder_order"#,
            Some(plist.rowid)
        )
            .map(Playlist::try_from)
            .fetch_all(&mut **tx)
            .await?;

        //子プレイリストの曲IDを追加していくSet
        let mut add_track_ids = HashSet::<i32>::new();

        for child in children {
            let child = child?;

            //子プレイリストの曲リストを取得
            let child_query = format!(
                "SELECT tracks.id {}",
                self.get_query_by_playlist(tx, &child).await?
            );
            let child_tracks: Vec<i32> = sqlx::query_scalar(&child_query)
                .fetch_all(&mut **tx)
                .await?;

            //Setに追加
            for track_id in child_tracks {
                add_track_ids.insert(track_id);
            }
        }

        Ok(add_track_ids.into_iter().collect())
    }

    /// プレイリストの設定に基づき、曲リストを取得：フィルタプレイリスト
    /// # Arguments
    /// - plist: 対象プレイリスト情報
    async fn search_plist_tracks_filter<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        plist: &Playlist,
    ) -> Result<Vec<i32>> {
        let filter = plist
            .filter
            .as_ref()
            .ok_or(Error::FilterPlaylistHasNoFilter {
                plist_id: plist.rowid,
            })?;

        self.track_lister_filter.list_track_id(tx, filter).await
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
        format!(" ORDER BY {}", order.replace("ASC", "DESC"))
    } else {
        format!(" ORDER BY {order}")
    }
}

/// カラムのソート順のクエリを取得
/// # Arguments
/// - sort_type: ソート対象
/// # Returns
/// order byに繋がる文字列。全ての列にasc付き
fn get_sort_column_query(sort_type: SortType) -> String {
    match sort_type {
	SortType::TrackName => "title_order ASC, tracks.id ASC".to_owned(),
	SortType::Artist => "artist_order ASC, album_order ASC, disc_number ASC, track_number ASC, title_order ASC, tracks.id ASC".to_owned(),
	SortType::Album => "album_order ASC, artist_order ASC, disc_number ASC, track_number ASC, title_order ASC, tracks.id ASC".to_owned(),
	SortType::Genre => "genre ASC, artist_order ASC, album_order ASC, disc_number ASC, track_number ASC, title_order ASC, trakcs.id ASC".to_owned(),
	SortType::Playlist => format!("[{PLIST_TRACK_IDX_COLUMN}] ASC"),
	SortType::Composer => "composer_order ASC, artist_order ASC, album_order ASC, disc_number ASC, track_number ASC, title_order ASC, tracks.id ASC".to_owned(),
	SortType::Duration => "duration ASC, title_order ASC, tracks.id ASC".to_owned(),
	SortType::TrackIndex => "track_number ASC, artist_order ASC, album_order ASC, disc_number ASC, title_order ASC, tracks.id ASC".to_owned(),
	SortType::DiscIndex => "disc_number ASC, artist_order ASC, album_order ASC, track_number ASC, title_order ASC, tracks.id ASC".to_owned(),
	SortType::ReleaseDate => "release_date ASC, artist_order ASC, album_order ASC, disc_number ASC, track_number ASC, title_order ASC, tracks.id ASC".to_owned(),
	SortType::Rating => "rating ASC, artist_order ASC, album_order ASC, disc_number ASC, track_number ASC, title_order ASC, tracks.id ASC".to_owned(),
	SortType::EntryDate => "created_at ASC, path ASC".to_owned(),
	SortType::Path => "path ASC".to_owned(),
	}
}
