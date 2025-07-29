//! DBとの連携データの読み書き機能

use std::{
    fs::{self, File},
    io::prelude::*,
    path::Path,
};

use anyhow::Result;
use murack_core_domain::{
    Error as DomainError, artwork::TrackArtwork, path::LibraryTrackPath, sync::TrackSync,
};
use murack_core_media::audio_meta::{AudioMetaData, FormatType, formats};

use crate::data_file::utils;

/// 曲のオーディオメタデータを読み込み
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - track_path: 取得対象の曲のライブラリ内パス
pub fn read_metadata(lib_root: &Path, track_path: &LibraryTrackPath) -> Result<AudioMetaData> {
    let track_abs = track_path.abs(lib_root);

    //ファイルがない場合はdomain側で判別したいので個別エラー
    if !track_abs.exists() {
        return Err(DomainError::FileTrackNotFound {
            lib_root: lib_root.to_owned(),
            track_path: track_path.to_owned(),
        }
        .into());
    }

    match FormatType::from_path(&track_abs)? {
        FormatType::Mp3 => formats::mp3::read(&track_abs),
        FormatType::Flac => formats::flac::read(&track_abs),
        FormatType::M4a => formats::m4a::read(&track_abs),
    }
}

/// DBと連携する曲データを読み込み
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - track_path: 取得対象の曲のライブラリ内パス
pub fn read_track_sync(lib_root: &Path, track_path: &LibraryTrackPath) -> Result<TrackSync> {
    let meta = read_metadata(lib_root, track_path)?;

    let track_abs = track_path.abs(lib_root);

    Ok(TrackSync {
        duration: meta.duration,
        title: meta.title.unwrap_or_default(),
        artist: meta.artist.unwrap_or_default(),
        album: meta.album.unwrap_or_default(),
        genre: meta.genre.unwrap_or_default(),
        album_artist: meta.album_artist.unwrap_or_default(),
        composer: meta.composer.unwrap_or_default(),
        track_number: meta.track_number,
        track_max: meta.track_max,
        disc_number: meta.disc_number,
        disc_max: meta.disc_max,
        release_date: meta.release_date,
        memo: meta.memo.unwrap_or_default(),
        artworks: meta.artworks.into_iter().map(TrackArtwork::from).collect(),
        lyrics: read_lyrics(&track_abs)?,
    })
}

/// DBと連携する曲データを上書き
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - track_path: 保存対象の曲のライブラリ内パス
/// - track_sync: 保存する曲データ
pub fn overwrite_track_sync(
    lib_root: &Path,
    track_path: &LibraryTrackPath,
    track_sync: &TrackSync,
) -> Result<()> {
    let track_abs = track_path.abs(lib_root);

    let (audio, artworks) = track_sync.get_audio_metadata_entry();

    match FormatType::from_path(&track_abs)? {
        FormatType::Mp3 => formats::mp3::overwrite(&track_abs, &audio, &artworks)?,
        FormatType::Flac => formats::flac::overwrite(&track_abs, &audio, &artworks)?,
        FormatType::M4a => formats::m4a::overwrite(&track_abs, &audio, &artworks)?,
    };

    write_lyrics(&track_abs, &track_sync.lyrics)?;

    Ok(())
}

/// オーディオファイルに対応する歌詞を読み込み
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
fn read_lyrics(path: &Path) -> Result<String> {
    let lrc_path = utils::get_lrc_path(path);

    let mut f = match File::open(lrc_path) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok(String::default());
            } else {
                return Err(io_to_my_error(path, e).into());
            }
        }
    };

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .map_err(|e| io_to_my_error(path, e))?;

    Ok(contents)
}

/// オーディオファイルに対応する歌詞を書き込み
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
/// - lyrics: 保存する歌詞
fn write_lyrics(path: &Path, lyrics: &str) -> Result<()> {
    let lrc_path = utils::get_lrc_path(path);

    if lyrics.is_empty() {
        // 歌詞が空の場合、.lrc ファイルがあれば削除
        if lrc_path.exists() {
            fs::remove_file(lrc_path).map_err(|e| io_to_my_error(path, e))?;
        }

        Ok(())
    } else {
        //歌詞が空でない場合
        fs::write(lrc_path, lyrics).map_err(|e| io_to_my_error(path, e))?;

        Ok(())
    }
}

/// std::io::error を DomainError::FileIoErrorに変換
fn io_to_my_error(path: &Path, e: std::io::Error) -> DomainError {
    DomainError::FileIoError(path.to_owned(), e)
}
