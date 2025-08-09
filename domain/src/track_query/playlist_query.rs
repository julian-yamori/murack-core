use std::collections::{BTreeSet, HashSet};

use async_recursion::async_recursion;
use sqlx::PgTransaction;
use sqlx::{Row, postgres::PgRow};

use crate::{
    NonEmptyString, SortType,
    path::LibraryTrackPath,
    playlist::{
        Playlist, PlaylistRow, PlaylistType, playlist_error::PlaylistError, playlist_tracks_sqls,
    },
    track_query::{TrackQueryError, filter_query},
};

/// playlist_track.orderカラムに付ける別名
const PLIST_TRACK_IDX_COLUMN: &str = "playlist_index";

/// プレイリストに含まれる曲のパスリストを取得
/// # Arguments
/// - plist 取得対象のプレイリスト情報(※childrenは不要)
pub async fn get_track_path_list<'c>(
    tx: &mut PgTransaction<'c>,
    plist: &Playlist,
) -> Result<Vec<LibraryTrackPath>, TrackQueryError> {
    //対象プレイリストのクエリ(from,join,where句)を取得
    let fjw_query = get_query_by_playlist(tx, plist).await?;

    //取得するカラム
    let mut clms_query = "tracks.path".to_owned();

    //プレイリスト順なら、取得カラムを一つ追加
    if plist.sort_type == SortType::Playlist {
        clms_query =
            format!("{clms_query}, playlist_tracks.order_index AS {PLIST_TRACK_IDX_COLUMN}");
    }

    let order_query = plist
        .sort_type
        .order_query(plist.sort_desc, PLIST_TRACK_IDX_COLUMN);

    //select句とorder byを結合
    let query = format!("SELECT {clms_query}{fjw_query} ORDER BY {order_query}",);

    let list: Vec<LibraryTrackPath> = sqlx::query(&query)
        .map(|row: PgRow| row.get::<LibraryTrackPath, _>(0))
        .fetch_all(&mut **tx)
        .await?;

    Ok(list)
}

/// プレイリストに含まれる曲を検索するクエリを作成
/// # Arguments
/// - plist: 対象プレイリスト情報
/// # Result
/// from,join,where句のクエリ
async fn get_query_by_playlist<'c>(
    tx: &mut PgTransaction<'c>,
    plist: &Playlist,
) -> Result<String, TrackQueryError> {
    //リストアップされていなければ、まずリストアップする
    if !plist.listuped_flag {
        listup_tracks(tx, plist).await?;
    }
    //プレイリストに含まれる曲の検索クエリを返す

    Ok(format!(
        " FROM playlist_trakcs JOIN tracks ON playlist_tracks.track_id = tracks.id WHERE playlist_tracks.playlist_id = {}",
        plist.id
    ))
}

/// プレイリストの曲をリストアップし、playlist_trackテーブルを更新する
/// # Arguments
/// - plist: 対象プレイリスト情報
async fn listup_tracks<'c>(
    tx: &mut PgTransaction<'c>,
    plist: &Playlist,
) -> Result<(), TrackQueryError> {
    //通常プレイリストなら、リストアップ済みフラグを立てるのみ
    if plist.playlist_type != PlaylistType::Normal {
        //元々保存されていた曲リストを取得
        let old_id_list: BTreeSet<_> =
            playlist_tracks_sqls::select_track_id_by_playlist_id(tx, plist.id)
                .await?
                .into_iter()
                .collect();

        let new_id_list = match plist.playlist_type {
            PlaylistType::Filter => search_plist_tracks_filter(tx, plist).await?,
            PlaylistType::Folder => search_plist_tracks_folder(tx, plist).await?,
            _ => unreachable!(),
        };

        //PlaylistTrackテーブルを更新
        playlist_tracks_sqls::delete_by_playlist_id(tx, plist.id).await?;
        for (idx, track_id) in new_id_list.iter().enumerate() {
            playlist_tracks_sqls::insert_playlist_track(tx, plist.id, *track_id, idx as i32)
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
                plist.id,
            )
            .execute(&mut **tx)
            .await?;
        }
    }

    //リストアップ済みに更新
    sqlx::query!(
        "UPDATE playlists SET listuped_flag = $1 WHERE id = $2",
        true,
        plist.id,
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
    tx: &mut PgTransaction<'c>,
    plist: &Playlist,
) -> Result<Vec<i32>, TrackQueryError> {
    let children = sqlx::query_as!(
            PlaylistRow,
            r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name AS "name: NonEmptyString", parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists WHERE parent_id IS NOT DISTINCT FROM $1 ORDER BY in_folder_order"#,
            Some(plist.id)
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
            get_query_by_playlist(tx, &child).await?
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
    tx: &mut PgTransaction<'c>,
    plist: &Playlist,
) -> Result<Vec<i32>, TrackQueryError> {
    let filter = plist
        .filter
        .as_ref()
        .ok_or(PlaylistError::FilterPlaylistHasNoFilter { plist_id: plist.id })?;

    filter_query::get_track_ids(tx, filter).await
}
