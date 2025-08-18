mod playlist_model;

use std::collections::{BTreeSet, HashSet};

use async_recursion::async_recursion;
use sqlx::PgTransaction;
use sqlx::postgres::PgRow;

use crate::{
    SortTypeWithPlaylist,
    playlist::{PlaylistType, playlist_error::PlaylistError, playlist_tracks_sqls},
    track_query::{
        SelectColumn, TrackQueryError, playlist_query::playlist_model::QueryPlaylistModel,
    },
};

/// プレイリストの曲の検索条件
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PlaylistQuery {
    /// 検索対象のプレイリストの ID
    playlist_id: i32,

    /// 取得するカラムの指定
    columns: Vec<SelectColumn>,

    /// LIMIT (曲レコードの取得件数)
    limit: Option<u32>,

    /// OFFSET (曲レコードの取得開始位置)
    offset: Option<u32>,
}

impl PlaylistQuery {
    /// カラムを指定し、プレイリストに含まれる曲を検索
    pub async fn fetch<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
    ) -> Result<Vec<PgRow>, TrackQueryError> {
        let plist = QueryPlaylistModel::from_db(tx, self.playlist_id).await?;

        //リストアップされていなければ、まず playlist_tracks テーブルを更新する
        if !plist.listuped_flag {
            update_playlist_tracks(tx, &plist).await?;
        }

        let mut join_queries = vec!["JOIN tracks ON playlist_tracks.track_id = tracks.id"];
        // アートワーク ID を取得する場合は、先頭のアートワークだけを取得できるように JOIN する
        if self.columns.contains(&SelectColumn::ArtworkId) {
            join_queries.push(
            "LEFT JOIN track_artworks ON track.id = track_artworks.track_id AND track_artworks.order_index = 0"
        )
        }

        let mut column_names: Vec<_> = self
            .columns
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

        // LIMIT, OFFSET が指定されていれば追加
        let limit_query = self
            .limit
            .map(|limit| format!("LIMIT {limit}"))
            .unwrap_or_default();
        let offset_query = self
            .offset
            .map(|offset| format!("OFFSET {offset}"))
            .unwrap_or_default();

        let sql = format!(
            "
            SELECT {}
            FROM playlist_tracks
            {}
            WHERE playlist_tracks.playlist_id = $1
            ORDER BY {order_query}
            {limit_query} {offset_query}",
            column_names.join(","),
            join_queries.join("\n")
        );
        let list: Vec<_> = sqlx::query(&sql)
            .bind(plist.id)
            .fetch_all(&mut **tx)
            .await?;

        Ok(list)
    }
}

#[derive(Debug, Clone)]
pub struct PlaylistQueryBuilder {
    playlist_id: i32,
    columns: Vec<SelectColumn>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl PlaylistQueryBuilder {
    pub fn new(playlist_id: i32) -> Self {
        Self {
            playlist_id,
            columns: Vec::default(),
            limit: None,
            offset: None,
        }
    }

    /// 取得するカラムを追加
    ///
    /// column は一つ以上の指定が必須
    pub fn column(mut self, column: SelectColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// `LIMIT` を指定 (曲レコードの取得件数)
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// `OFFSET` を指定 (曲レコードの取得開始位置)
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build(self) -> PlaylistQuery {
        assert!(!self.columns.is_empty(), "columns cannot be empty");

        PlaylistQuery {
            playlist_id: self.playlist_id,
            columns: self.columns,
            limit: self.limit,
            offset: self.offset,
        }
    }
}

/// プレイリストの曲をリストアップし、playlist_trackテーブルを更新する
/// # Arguments
/// - plist: 対象プレイリスト情報
async fn update_playlist_tracks<'c>(
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
        //リストアップされていなければ、まず playlist_tracks テーブルを更新する
        if !child.listuped_flag {
            update_playlist_tracks(tx, &child).await?;
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
async fn search_plist_tracks_filter<'c>(
    tx: &mut PgTransaction<'c>,
    plist: &QueryPlaylistModel,
) -> Result<Vec<i32>, TrackQueryError> {
    let filter = plist
        .filter
        .as_ref()
        .ok_or(PlaylistError::FilterPlaylistHasNoFilter { plist_id: plist.id })?;

    let mut query_base = "SELECT tracks.id FROM tracks".to_owned();

    //フィルタから条件を取得して追加
    if let Some(query_where) = filter.where_expression() {
        query_base = format!("{query_base} WHERE {query_where}");
    }

    let list = sqlx::query_scalar(&query_base).fetch_all(&mut **tx).await?;

    Ok(list)
}
