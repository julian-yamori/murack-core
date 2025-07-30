use anyhow::Result;
use sqlx::PgTransaction;

use crate::playlist::PlaylistType;

/// 全フィルタプレイリスト・フォルダプレイリストの、リストアップ済みフラグを解除する。
pub async fn reset_listuped_flag<'c>(tx: &mut PgTransaction<'c>) -> Result<()> {
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
pub async fn set_dap_change_flag_all<'c>(
    tx: &mut PgTransaction<'c>,
    is_changed: bool,
) -> Result<()> {
    sqlx::query!("UPDATE playlists SET dap_changed = $1", is_changed,)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

/// プレイリストIDを指定して曲IDを取得
pub async fn select_track_id_by_playlist_id<'c>(
    tx: &mut PgTransaction<'c>,
    plist_id: i32,
) -> Result<Vec<i32>> {
    let id = sqlx::query_scalar!(
        "SELECT track_id FROM playlist_tracks WHERE playlist_id = $1",
        plist_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(id)
}

/// プレイリストに曲を新規登録
pub async fn insert_playlist_track<'c>(
    tx: &mut PgTransaction<'c>,
    plist_id: i32,
    track_id: i32,
    order: i32,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO playlist_tracks (playlist_id, order_index, track_id) VALUES($1, $2, $3)",
        plist_id,
        order,
        track_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// プレイリストIDを指定して削除
///
/// # Arguments
/// - plist_id: 削除元のプレイリストのID
pub async fn delete_by_playlist_id<'c>(tx: &mut PgTransaction<'c>, plist_id: i32) -> Result<()> {
    sqlx::query!(
        "DELETE FROM playlist_tracks WHERE playlist_id = $1",
        plist_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
