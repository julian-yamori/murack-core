//! 曲データのコピー機能

use std::{fs, path::Path};

use anyhow::{Context, Result};
use murack_core_domain::{Error as DomainError, path::LibraryTrackPath};

use crate::data_file::{Error, utils};

/// ライブラリからライブラリへ、曲データをコピー
///
/// コピー先にファイルが既に存在する場合はエラーとする。
///
/// # Arguments
/// - src_lib: コピー元のライブラリのルート絶対パス
/// - dest_lib: コピー先のライブラリのルート絶対パス
/// - target: コピーする曲のライブラリ内パス
pub fn copy_track_over_lib(
    src_lib: &Path,
    dest_lib: &Path,
    target: &LibraryTrackPath,
) -> Result<()> {
    //コピー元にファイルがあるか確認
    let src_track = target.abs(src_lib);
    if !src_track.exists() {
        return Err(DomainError::FileTrackNotFound {
            lib_root: src_lib.to_owned(),
            track_path: target.to_owned(),
        }
        .into());
    }

    //コピー先に既に存在しないか確認
    let dest_track = target.abs(dest_lib);
    if dest_track.exists() {
        return Err(DomainError::FileTrackAlreadyExists {
            lib_root: dest_lib.to_owned(),
            track_path: target.to_owned(),
        }
        .into());
    }

    //コピー先で不足しているディレクトリを作成
    if let Some(parent) = dest_track.parent() {
        fs::create_dir_all(parent).with_context(|| parent.display().to_string())?;
    }

    //コピーを実行
    copy(&src_track, &dest_track)?;

    let src_lrc = utils::get_lrc_path(&src_track);

    //コピー元に歌詞ファイルがある場合、コピー
    if src_lrc.exists() {
        let dest_lrc = utils::get_lrc_path(&dest_track);
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
/// - track_path: コピー先のライブラリ内パス
/// - src_path: コピー元のライブラリ外のファイル絶対パス
pub fn copy_from_outside_lib(
    lib_root: &Path,
    track_path: &LibraryTrackPath,
    src_path: &Path,
) -> Result<()> {
    //コピー元にファイルがあるか確認
    if !src_path.exists() {
        return Err(Error::AbsFileNotFound(src_path.to_owned()).into());
    }

    //コピー先に既に存在しないか確認
    let dest_track = track_path.abs(lib_root);
    if dest_track.exists() {
        return Err(DomainError::FileTrackAlreadyExists {
            lib_root: lib_root.to_owned(),
            track_path: track_path.to_owned(),
        }
        .into());
    }

    //コピー先で不足しているディレクトリを作成
    if let Some(parent) = dest_track.parent() {
        fs::create_dir_all(parent).with_context(|| parent.display().to_string())?;
    }

    //コピーを実行
    copy(src_path, &dest_track)
}

/// ライブラリからライブラリへ、曲データを上書き
///
/// # Arguments
/// - src_lib: コピー元のライブラリのルート絶対パス
/// - dest_lib: 上書き先のライブラリのルート絶対パス
/// - target: コピーする曲のライブラリ内パス
pub fn overwrite_track_over_lib(
    src_lib: &Path,
    dest_lib: &Path,
    target: &LibraryTrackPath,
) -> Result<()> {
    //コピー元にファイルがあるか確認
    let src_track = target.abs(src_lib);
    if !src_track.exists() {
        return Err(DomainError::FileTrackNotFound {
            lib_root: src_lib.to_owned(),
            track_path: target.to_owned(),
        }
        .into());
    }

    //上書きコピーを実行
    let dest_track = target.abs(dest_lib);
    copy(&src_track, &dest_track)?;

    let src_lrc = utils::get_lrc_path(&src_track);
    let dest_lrc = utils::get_lrc_path(&dest_track);

    if src_lrc.exists() {
        //コピー元に歌詞ファイルがある場合、コピー
        copy(&src_lrc, &dest_lrc)?;
    } else {
        //コピー元に歌詞ファイルがない場合
        //上書き先にあれば削除
        if dest_lrc.exists() {
            fs::remove_file(&dest_lrc).with_context(|| dest_lrc.display().to_string())?;
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
