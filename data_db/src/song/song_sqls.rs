use anyhow::Result;
use murack_core_domain::{
    db::DbTransaction,
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibSongPath},
};
use sqlx::{Row, postgres::PgRow};

use super::SongEntry;
use crate::{converts::enums::db_from_folder_id_may_root, like_esc};

/// パスを指定してrowidを取得
pub async fn select_id_by_path<'c>(
    tx: &mut DbTransaction<'c>,
    path: &LibSongPath,
) -> Result<Option<i32>> {
    let id = sqlx::query_scalar!("SELECT id FROM tracks WHERE path = $1", path.as_str(),)
        .fetch_optional(&mut **tx.get())
        .await?;

    Ok(id)
}

/// 全レコードのパスを取得
pub async fn select_path_all<'c>(tx: &mut DbTransaction<'c>) -> Result<Vec<LibSongPath>> {
    let paths = sqlx::query!("SELECT path FROM tracks",)
        .map(|row| LibSongPath::new(row.path))
        .fetch_all(&mut **tx.get())
        .await?;

    Ok(paths)
}

/// 指定されたディレクトリで始まるパスを取得
pub async fn select_path_begins_directory<'c>(
    tx: &mut DbTransaction<'c>,
    path: &LibDirPath,
) -> Result<Vec<LibSongPath>> {
    let path_str = path.as_str();

    //LIKE文エスケープ
    let cmp_value_buff;
    let (like_query, cmp_value) = if like_esc::is_need(path_str) {
        cmp_value_buff = like_esc::escape(path_str);
        ("LIKE $1 || '%' ESCAPE '$'", cmp_value_buff.as_str())
    } else {
        ("LIKE $1 || '%'", path_str)
    };

    let sql = format!("SELECT path FROM tracks WHERE path {like_query}");
    let paths = sqlx::query(&sql)
        .bind(cmp_value)
        .map(|row: PgRow| LibSongPath::new(row.get::<&str, _>(0)))
        .fetch_all(&mut **tx.get())
        .await?;

    Ok(paths)
}

/// 指定されたpathのレコードが存在するか確認
pub async fn exists_path<'c>(tx: &mut DbTransaction<'c>, path: &LibSongPath) -> Result<bool> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE path = $1"#,
        path.as_str(),
    )
    .fetch_one(&mut **tx.get())
    .await?;

    Ok(count > 0)
}

/// 新規登録
///
/// # Returns
/// 登録されたレコードのrowid
pub async fn insert<'c, 'e>(tx: &mut DbTransaction<'c>, entry: &SongEntry<'e>) -> Result<i32> {
    let id = sqlx::query_scalar!(
        "INSERT INTO tracks (duration, path, folder_id, title, artist, album, genre, album_artist, composer, track_number, track_max, disc_number, disc_max, release_date, rating, original_track, suggest_target, memo, memo_manage, lyrics, title_order, artist_order, album_order, album_artist_order, composer_order, genre_order) values($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26) RETURNING id",
        entry.duration,
        entry.path,
        entry.folder_id,
        entry.title,
        entry.artist,
        entry.album,
        entry.genre,
        entry.album_artist,
        entry.composer,
        entry.track_number,
        entry.track_max,
        entry.disc_number,
        entry.disc_max,
        entry.release_date,
        entry.rating,
        entry.original_song,
        entry.suggest_target,
        entry.memo,
        entry.memo_manage,
        entry.lyrics,
        entry.title_order,
        entry.artist_order,
        entry.album_order,
        entry.album_artist_order,
        entry.composer_order,
        entry.genre_order,
    ).fetch_one(&mut **tx.get()).await?;

    Ok(id)
}

/// IDを指定してレコードを削除
pub async fn delete<'c>(tx: &mut DbTransaction<'c>, song_id: i32) -> Result<()> {
    sqlx::query!("DELETE FROM tracks WHERE id = $1", song_id,)
        .execute(&mut **tx.get())
        .await?;

    Ok(())
}

/// フォルダIDを指定して曲数を取得
pub async fn count_by_folder_id<'c>(
    tx: &mut DbTransaction<'c>,
    folder_id: FolderIdMayRoot,
) -> Result<i64> {
    let folder_id_value = db_from_folder_id_may_root(folder_id);
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE folder_id IS NOT DISTINCT FROM $1"#,
        folder_id_value,
    )
    .fetch_one(&mut **tx.get())
    .await?;

    Ok(count)
}

/// 更新対象を旧パスで指定し、新しいパス情報で更新
pub async fn update_path_by_path<'c>(
    tx: &mut DbTransaction<'c>,
    old_path: &LibSongPath,
    new_path: &LibSongPath,
    new_folder_id: FolderIdMayRoot,
) -> Result<()> {
    let folder_id_value = db_from_folder_id_may_root(new_folder_id);

    sqlx::query!(
        "UPDATE tracks SET path = $1, folder_id = $2 WHERE path = $3",
        new_path.as_str(),
        folder_id_value,
        old_path.as_str(),
    )
    .execute(&mut **tx.get())
    .await?;

    Ok(())
}

/// IDと再生時間を指定して再生時間を更新
pub async fn update_duration_by_id<'c>(
    tx: &mut DbTransaction<'c>,
    song_id: i32,
    duration: u32,
) -> Result<()> {
    let duration_i32: i32 = duration.try_into()?;

    sqlx::query!(
        "UPDATE tracks SET duration = $1 WHERE id = $2",
        duration_i32,
        song_id,
    )
    .execute(&mut **tx.get())
    .await?;

    Ok(())
}
