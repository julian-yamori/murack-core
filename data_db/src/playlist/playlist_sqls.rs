use anyhow::Result;
use murack_core_domain::{
    db::DbTransaction,
    playlist::{PlaylistType, SortType},
};

use super::PlaylistRow;

/// プレイリストIDを指定して曲IDを取得
pub async fn select_song_id_by_playlist_id<'c>(
    tx: &mut DbTransaction<'c>,
    plist_id: i32,
) -> Result<Vec<i32>> {
    let id = sqlx::query_scalar!(
        "SELECT track_id FROM playlist_tracks WHERE playlist_id = $1",
        plist_id,
    )
    .fetch_all(&mut **tx.get())
    .await?;

    Ok(id)
}

/// プレイリストに曲を新規登録
pub async fn insert_playlist_song<'c>(
    tx: &mut DbTransaction<'c>,
    plist_id: i32,
    song_id: i32,
    order: i32,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO playlist_tracks (playlist_id, order_index, track_id) VALUES($1, $2, $3)",
        plist_id,
        order,
        song_id,
    )
    .execute(&mut **tx.get())
    .await?;

    Ok(())
}

/// プレイリストIDを指定して削除
///
/// # Arguments
/// - plist_id: 削除元のプレイリストのID
pub async fn delete_by_playlist_id<'c>(tx: &mut DbTransaction<'c>, plist_id: i32) -> Result<()> {
    sqlx::query!(
        "DELETE FROM playlist_tracks WHERE playlist_id = $1",
        plist_id,
    )
    .execute(&mut **tx.get())
    .await?;

    Ok(())
}

/// IDを指定してプレイリストを検索
pub async fn select_playlist_by_id<'c>(
    tx: &mut DbTransaction<'c>,
    plist_id: i32,
) -> Result<Option<PlaylistRow>> {
    let row = sqlx::query_as!(
        PlaylistRow,
        r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name, parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists WHERE id = $1"#,
        plist_id
    )
    .fetch_optional(&mut **tx.get())
    .await?;

    Ok(row)
}

/// 全プレイリストを取得
pub async fn select_all_playlists<'c>(tx: &mut DbTransaction<'c>) -> Result<Vec<PlaylistRow>> {
    let rows = sqlx::query_as!(PlaylistRow, r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name, parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists"#)
        .fetch_all(&mut **tx.get())
        .await?;

    Ok(rows)
}

/// 全プレイリストを取得(in_folder_order順)
pub async fn select_all_playlists_order_folder<'c>(
    tx: &mut DbTransaction<'c>,
) -> Result<Vec<PlaylistRow>> {
    let rows = sqlx::query_as!(
        PlaylistRow,
        r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name, parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists ORDER BY in_folder_order"#
    )
    .fetch_all(&mut **tx.get())
    .await?;

    Ok(rows)
}

/// プレイリストの子プレイリスト一覧を取得
/// # Arguments
/// - parent_id: 親プレイリストID(Noneなら最上位のプレイリストを取得)
/// # Returns
/// 指定されたプレイリストの子プレイリスト一覧
pub async fn get_child_playlists<'c>(
    tx: &mut DbTransaction<'c>,
    parent_id: Option<i32>,
) -> Result<Vec<PlaylistRow>> {
    let rows = sqlx::query_as!(PlaylistRow, r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name, parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists WHERE parent_id IS NOT DISTINCT FROM $1 ORDER BY in_folder_order"#, parent_id).fetch_all(&mut **tx.get()).await?;

    Ok(rows)
}
