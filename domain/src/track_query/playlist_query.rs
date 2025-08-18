mod playlist_model;

use std::collections::{BTreeSet, HashSet};

use async_recursion::async_recursion;
use sqlx::PgTransaction;
use sqlx::postgres::PgRow;

use crate::{
    SortTypeWithPlaylist,
    playlist::{PlaylistType, playlist_error::PlaylistError, playlist_tracks_sqls},
    track_query::{
        SelectColumn, TrackQueryError, filter_query,
        playlist_query::playlist_model::QueryPlaylistModel,
    },
};

/// カラムを指定し、プレイリストに含まれる曲を検索
pub async fn select_tracks<'c>(
    tx: &mut PgTransaction<'c>,
    playlist_id: i32,
    columns: impl Iterator<Item = SelectColumn>,
) -> Result<Vec<PgRow>, TrackQueryError> {
    let plist = QueryPlaylistModel::from_db(tx, playlist_id).await?;

    //リストアップされていなければ、まずリストアップする
    if !plist.listuped_flag {
        listup_tracks(tx, &plist).await?;
    }

    let columns_set: HashSet<_> = columns.collect();

    let mut join_queries = vec!["JOIN tracks ON playlist_tracks.track_id = tracks.id"];
    // アートワーク ID を取得する場合は、先頭のアートワークだけを取得できるように JOIN する
    if columns_set.contains(&SelectColumn::ArtworkId) {
        join_queries.push(
            "LEFT JOIN track_artworks ON track.id = track_artworks.track_id AND track_artworks.order_index = 0"
        )
    }

    let mut column_names: Vec<_> = columns_set
        .iter()
        .map(SelectColumn::sql_column_name)
        .collect();
    //プレイリスト順なら、取得カラムを一つ追加
    if plist.sort_type == SortTypeWithPlaylist::Playlist {
        column_names.push("playlist_tracks.order_index");
    }

    let order_query = plist
        .sort_type
        .order_query(plist.sort_desc, "playlist_tracks.order_index");

    let sql = format!(
        "
        SELECT {}
        FROM playlist_tracks
        {}
        WHERE playlist_tracks.playlist_id = $1
        ORDER BY {order_query}
        ",
        column_names.join(","),
        join_queries.join("\n")
    );
    let list: Vec<_> = sqlx::query(&sql)
        .bind(plist.id)
        .fetch_all(&mut **tx)
        .await?;

    Ok(list)
}

/// プレイリストの曲をリストアップし、playlist_trackテーブルを更新する
/// # Arguments
/// - plist: 対象プレイリスト情報
async fn listup_tracks<'c>(
    tx: &mut PgTransaction<'c>,
    plist: &QueryPlaylistModel,
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
    plist: &QueryPlaylistModel,
) -> Result<Vec<i32>, TrackQueryError> {
    //直下の子のプレイリストを取得
    let children = QueryPlaylistModel::from_db_by_parent(tx, plist.id).await?;

    //子プレイリストの曲IDを追加していくSet
    let mut add_track_ids = HashSet::<i32>::new();

    for child in children {
        //リストアップされていなければ、まずリストアップする
        if !child.listuped_flag {
            listup_tracks(tx, &child).await?;
        }

        //子プレイリストの曲リストを取得
        let child_tracks: Vec<i32> = sqlx::query_scalar!(
            "
            SELECT tracks.id
            FROM playlist_tracks
            JOIN tracks
              ON playlist_tracks.track_id = tracks.id
            WHERE playlist_tracks.playlist_id = $1
            ",
            child.id
        )
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
    plist: &QueryPlaylistModel,
) -> Result<Vec<i32>, TrackQueryError> {
    let filter = plist
        .filter
        .as_ref()
        .ok_or(PlaylistError::FilterPlaylistHasNoFilter { plist_id: plist.id })?;

    filter_query::get_track_ids(tx, filter).await
}
