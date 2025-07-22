//! DBとの連携データの読み書き機能

use std::{
    fs::{self, File},
    io::prelude::*,
    path::Path,
};

use anyhow::Result;
use murack_core_domain::{
    Error as DomainError, artwork::SongArtwork, path::LibSongPath, sync::SongSync,
};
use murack_core_media::audio_meta::{AudioMetaData, FormatType, formats};

use crate::utils;

/// 曲のオーディオメタデータを読み込み
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - song_path: 取得対象の曲のライブラリ内パス
pub fn read_metadata(lib_root: &Path, song_path: &LibSongPath) -> Result<AudioMetaData> {
    let song_abs = song_path.abs(lib_root);

    //ファイルがない場合はdomain側で判別したいので個別エラー
    if !song_abs.exists() {
        return Err(DomainError::FileSongNotFound {
            lib_root: lib_root.to_owned(),
            song_path: song_path.to_owned(),
        }
        .into());
    }

    match FormatType::from_path(&song_abs)? {
        FormatType::Mp3 => formats::mp3::read(&song_abs),
        FormatType::Flac => formats::flac::read(&song_abs),
        FormatType::M4a => formats::m4a::read(&song_abs),
    }
}

/// DBと連携する曲データを読み込み
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - song_path: 取得対象の曲のライブラリ内パス
pub fn read(lib_root: &Path, song_path: &LibSongPath) -> Result<SongSync> {
    let meta = read_metadata(lib_root, song_path)?;

    let song_abs = song_path.abs(lib_root);

    Ok(SongSync {
        duration: meta.duration,
        title: meta.title,
        artist: meta.artist,
        album: meta.album,
        genre: meta.genre,
        album_artist: meta.album_artist,
        composer: meta.composer,
        track_number: meta.track_number,
        track_max: meta.track_max,
        disc_number: meta.disc_number,
        disc_max: meta.disc_max,
        release_date: meta.release_date,
        memo: meta.memo,
        artworks: meta.artworks.into_iter().map(SongArtwork::from).collect(),
        lyrics: read_lyrics(&song_abs)?,
    })
}

/// DBと連携する曲データを上書き
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - song_path: 保存対象の曲のライブラリ内パス
/// - song_sync: 保存する曲データ
pub fn overwrite(lib_root: &Path, song_path: &LibSongPath, song_sync: &SongSync) -> Result<()> {
    let song_abs = song_path.abs(lib_root);

    let (audio, artworks) = song_sync.get_audio_metadata_entry();

    match FormatType::from_path(&song_abs)? {
        FormatType::Mp3 => formats::mp3::overwrite(&song_abs, &audio, &artworks)?,
        FormatType::Flac => formats::flac::overwrite(&song_abs, &audio, &artworks)?,
        FormatType::M4a => formats::m4a::overwrite(&song_abs, &audio, &artworks)?,
    };

    write_lyrics(&song_abs, &song_sync.lyrics)?;

    Ok(())
}

/// オーディオファイルに対応する歌詞を読み込み
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
fn read_lyrics(path: &Path) -> Result<Option<String>> {
    let lrc_path = utils::get_lrc_path(path);

    let mut f = match File::open(lrc_path) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok(None);
            } else {
                return Err(io_to_my_error(path, e).into());
            }
        }
    };

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .map_err(|e| io_to_my_error(path, e))?;

    if contents.is_empty() {
        Ok(None)
    } else {
        Ok(Some(contents))
    }
}

/// オーディオファイルに対応する歌詞を書き込み
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
/// - lyrics: 保存する歌詞
fn write_lyrics(path: &Path, lyrics: &Option<String>) -> Result<()> {
    let lrc_path = utils::get_lrc_path(path);

    match lyrics {
        //歌詞が空でない場合
        Some(lyrics_str) => {
            fs::write(lrc_path, lyrics_str).map_err(|e| io_to_my_error(path, e))?;

            Ok(())
        }
        //歌詞が空の場合
        None => {
            //lrcファイルがあれば削除
            if lrc_path.exists() {
                fs::remove_file(lrc_path).map_err(|e| io_to_my_error(path, e))?;
            }

            Ok(())
        }
    }
}

/// std::io::error を DomainError::FileIoErrorに変換
fn io_to_my_error(path: &Path, e: std::io::Error) -> DomainError {
    DomainError::FileIoError(path.to_owned(), e)
}
