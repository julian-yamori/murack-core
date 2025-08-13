#[cfg(test)]
mod tests;

use anyhow::Result;
use murack_core_domain::{
    folder::{FolderIdMayRoot, folder_repository},
    path::LibraryTrackPath,
    playlist::playlist_sqls,
};
use sqlx::PgTransaction;

use crate::{
    DbTrackError, app_artwork_repository, db_common,
    track_data::AudioMetadata,
    track_sync::{DbTrackSync, TrackSyncRow},
};

/// パスを指定して曲情報を取得
///
/// # Arguments
/// - path 曲のパス
/// # Returns
/// 該当する曲の情報（見つからない場合はNone）
pub async fn get_by_path<'c>(
    tx: &mut PgTransaction<'c>,
    path: &LibraryTrackPath,
) -> Result<Option<DbTrackSync>> {
    //一旦trackテーブルから検索
    let track_row = match sqlx::query_as!(
        TrackSyncRow,
        "SELECT id, duration, title, artist, album, genre, album_artist, composer, track_number, track_max, disc_number, disc_max, release_date, memo, lyrics FROM tracks WHERE path = $1",
        path.as_ref() as &str
    ).fetch_optional(&mut **tx).await? {
        Some(t) => t,
        None => return Ok(None),
    };

    Ok(Some(DbTrackSync {
        id: track_row.id,
        path: path.clone(),
        metadata: AudioMetadata {
            duration: track_row.duration.try_into()?,
            title: track_row.title,
            artist: track_row.artist,
            album: track_row.album,
            genre: track_row.genre,
            album_artist: track_row.album_artist,
            composer: track_row.composer,
            track_number: track_row.track_number,
            track_max: track_row.track_max,
            disc_number: track_row.disc_number,
            disc_max: track_row.disc_max,
            release_date: track_row.release_date,
            memo: track_row.memo,
            lyrics: track_row.lyrics,
            //アートワーク情報を検索して紐づけ
            artworks: app_artwork_repository::get_track_artworks(tx, track_row.id).await?,
        },
    }))
}

/// DBに曲データを新規登録する
///
/// # Arguments
/// - db: DB接続
/// - track_path: 登録する曲のライブラリ内パス
/// - metadata: 登録する曲のデータ
pub async fn register_db<'c>(
    tx: &mut PgTransaction<'c>,
    track_path: &LibraryTrackPath,
    metadata: AudioMetadata,
) -> Result<()> {
    //DBに既に存在しないか確認
    if db_common::exists_path(tx, track_path).await? {
        return Err(DbTrackError::DbTrackAlreadyExists(track_path.clone()).into());
    }

    //親ディレクトリを登録してIDを取得
    let parent_path_opt = track_path.parent();
    let folder_id = match parent_path_opt {
        None => FolderIdMayRoot::Root,
        Some(parent_path) => {
            let id = folder_repository::register_not_exists(tx, &parent_path).await?;
            FolderIdMayRoot::Folder(id)
        }
    };

    // tracks テーブルに書き込み
    let track_id = sqlx::query_scalar!(
        "INSERT INTO tracks (duration, path, folder_id, title, artist, album, genre, album_artist, composer, track_number, track_max, disc_number, disc_max, release_date, rating, original_track, suggest_target, memo, memo_manage, lyrics, title_order, artist_order, album_order, album_artist_order, composer_order, genre_order) values($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26) RETURNING id",
        i32::try_from(metadata.duration)?,
        track_path.as_ref() as &str,
        folder_id.into_db(),
        &metadata.title,
        &metadata.artist,
        &metadata.album,
        &metadata.genre,
        &metadata.album_artist,
        &metadata.composer,
        metadata.track_number,
        metadata.track_max,
        metadata.disc_number,
        metadata.disc_max,
        metadata.release_date,
        0, // rating
        "", // original_track
        true, // suggest_target
        &metadata.memo,
        "", // memo_manage,
        &metadata.lyrics,
        metadata.title_order(),
        metadata.artist_order(),
        metadata.album_order(),
        metadata.album_artist_order(),
        metadata.composer_order(),
        metadata.genre_order(),
    ).fetch_one(&mut **tx).await?;

    //アートワークを登録
    app_artwork_repository::register_track_artworks(tx, track_id, metadata.artworks).await?;

    //プレイリストのリストアップ済みフラグを解除
    playlist_sqls::reset_listuped_flag(tx).await?;

    Ok(())
}

/// 曲の連携情報をDBに保存(アートワーク以外)
///
/// アートワークは重すぎるので除外。
/// ArtworkRepositoryの保存処理を直接呼び出すこと。
pub async fn save_exclude_artwork<'c>(
    tx: &mut PgTransaction<'c>,
    track: &DbTrackSync,
) -> Result<()> {
    let sync = &track.metadata;

    // duration を i32 に変換
    let duration: i32 = sync.duration.try_into()?;

    sqlx::query!(
        "UPDATE tracks SET duration = $1, title = $2, artist = $3, album = $4, genre = $5, album_artist = $6, composer = $7, track_number = $8, track_max = $9, disc_number = $10, disc_max = $11, release_date = $12, memo = $13, lyrics = $14, title_order = $15, artist_order = $16, album_order = $17, album_artist_order = $18, composer_order = $19, genre_order = $20 WHERE id = $21",
        duration,
        &sync.title,
        &sync.artist,
        &sync.album,
        &sync.genre,
        &sync.album_artist,
        &sync.composer,
        sync.track_number,
        sync.track_max,
        sync.disc_number,
        sync.disc_max,
        sync.release_date,
        &sync.memo,
        &sync.lyrics,
        sync.title_order(),
        sync.artist_order(),
        sync.album_order(),
        sync.album_artist_order(),
        sync.composer_order(),
        sync.genre_order(),
        track.id,
    ).execute(&mut **tx).await?;

    Ok(())
}
