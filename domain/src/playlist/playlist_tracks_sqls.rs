use sqlx::PgTransaction;

/// プレイリストIDを指定して曲IDを取得
pub async fn select_track_id_by_playlist_id<'c>(
    tx: &mut PgTransaction<'c>,
    plist_id: i32,
) -> sqlx::Result<Vec<i32>> {
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
) -> sqlx::Result<()> {
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
pub async fn delete_by_playlist_id<'c>(
    tx: &mut PgTransaction<'c>,
    plist_id: i32,
) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM playlist_tracks WHERE playlist_id = $1",
        plist_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
