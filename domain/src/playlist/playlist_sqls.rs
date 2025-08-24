use sqlx::PgTransaction;

use crate::playlist::PlaylistType;

/// 全フィルタプレイリスト・フォルダプレイリストの、リストアップ済みフラグを解除する。
pub async fn reset_listuped_flag<'c>(tx: &mut PgTransaction<'c>) -> sqlx::Result<()> {
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

pub async fn set_dap_changed<'c>(
    tx: &mut PgTransaction<'c>,
    playlist_id: i32,
    is_changed: bool,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE playlists SET dap_changed = $1 WHERE id = $2",
        is_changed,
        playlist_id
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
) -> sqlx::Result<()> {
    sqlx::query!("UPDATE playlists SET dap_changed = $1", is_changed,)
        .execute(&mut **tx)
        .await?;

    Ok(())
}
