//! 曲データのコピー機能

use crate::{Error, utils};
use anyhow::{Context, Result};
use domain::path::LibSongPath;
use std::{fs, path::Path};

/// ライブラリからライブラリへ、曲データをコピー
///
/// コピー先にファイルが既に存在する場合はエラーとする。
///
/// # Arguments
/// - src_lib: コピー元のライブラリのルート絶対パス
/// - dest_lib: コピー先のライブラリのルート絶対パス
/// - target: コピーする曲のライブラリ内パス
pub fn copy_song_over_lib(src_lib: &Path, dest_lib: &Path, target: &LibSongPath) -> Result<()> {
    //コピー元にファイルがあるか確認
    let src_song = target.abs(src_lib);
    if !src_song.exists() {
        return Err(domain::Error::FileSongNotFound {
            lib_root: src_lib.to_owned(),
            song_path: target.to_owned(),
        }
        .into());
    }

    //コピー先に既に存在しないか確認
    let dest_song = target.abs(dest_lib);
    if dest_song.exists() {
        return Err(domain::Error::FileSongAlreadyExists {
            lib_root: dest_lib.to_owned(),
            song_path: target.to_owned(),
        }
        .into());
    }

    //コピー先で不足しているディレクトリを作成
    if let Some(parent) = dest_song.parent() {
        fs::create_dir_all(parent).map_err(|e| domain::Error::FileIoError(parent.to_owned(), e))?;
    }

    //コピーを実行
    copy(&src_song, &dest_song)?;

    let src_lrc = utils::get_lrc_path(&src_song);

    //コピー元に歌詞ファイルがある場合、コピー
    if src_lrc.exists() {
        let dest_lrc = utils::get_lrc_path(&dest_song);
        copy(&src_lrc, &dest_lrc)?;
    }

    Ok(())
}

/// ライブラリ外からライブラリ内にファイルをコピー
///
/// lrcファイルは扱わない
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - song_path: コピー先のライブラリ内パス
/// - src_path: コピー元のライブラリ外のファイル絶対パス
pub fn copy_from_outside_lib(
    lib_root: &Path,
    song_path: &LibSongPath,
    src_path: &Path,
) -> Result<()> {
    //コピー元にファイルがあるか確認
    if !src_path.exists() {
        return Err(Error::AbsFileNotFound(src_path.to_owned()).into());
    }

    //コピー先に既に存在しないか確認
    let dest_song = song_path.abs(lib_root);
    if dest_song.exists() {
        return Err(domain::Error::FileSongAlreadyExists {
            lib_root: lib_root.to_owned(),
            song_path: song_path.to_owned(),
        }
        .into());
    }

    //コピー先で不足しているディレクトリを作成
    if let Some(parent) = dest_song.parent() {
        fs::create_dir_all(parent).map_err(|e| domain::Error::FileIoError(parent.to_owned(), e))?;
    }

    //コピーを実行
    copy(src_path, &dest_song)
}

/// ライブラリからライブラリへ、曲データを上書き
///
/// # Arguments
/// - src_lib: コピー元のライブラリのルート絶対パス
/// - dest_lib: 上書き先のライブラリのルート絶対パス
/// - target: コピーする曲のライブラリ内パス
pub fn overwrite_song_over_lib(
    src_lib: &Path,
    dest_lib: &Path,
    target: &LibSongPath,
) -> Result<()> {
    //コピー元にファイルがあるか確認
    let src_song = target.abs(src_lib);
    if !src_song.exists() {
        return Err(domain::Error::FileSongNotFound {
            lib_root: src_lib.to_owned(),
            song_path: target.to_owned(),
        }
        .into());
    }

    //上書きコピーを実行
    let dest_song = target.abs(dest_lib);
    copy(&src_song, &dest_song)?;

    let src_lrc = utils::get_lrc_path(&src_song);
    let dest_lrc = utils::get_lrc_path(&dest_song);

    if src_lrc.exists() {
        //コピー元に歌詞ファイルがある場合、コピー
        copy(&src_lrc, &dest_lrc)?;
    } else {
        //コピー元に歌詞ファイルがない場合
        //上書き先にあれば削除
        if dest_lrc.exists() {
            fs::remove_file(&dest_lrc).map_err(|e| domain::Error::FileIoError(dest_lrc, e))?;
        }
    }

    Ok(())
}

/// ファイルのコピー
fn copy(from: &Path, to: &Path) -> Result<()> {
    fs::copy(from, to).with_context(|| {
        format!(
            "failed to copy file. from: {} to: {}",
            from.display(),
            to.display()
        )
    })?;

    Ok(())
}
