use anyhow::Result;
use murack_core_domain::db::DbTransaction;

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
