//! 曲データの削除機能

use std::fs;
use std::path::Path;

use anyhow::Result;
use murack_core_domain::{Error as DomainError, NonEmptyString, path::LibraryTrackPath};

use crate::data_file::utils;

/// ライブラリから曲を削除
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - target: 削除対象の曲のライブラリ内パス
pub fn delete_track(lib_root: &Path, target: &LibraryTrackPath) -> Result<()> {
    //ファイルが存在するか確認
    let abs_path = target.abs(lib_root);
    if !abs_path.exists() {
        return Err(DomainError::FileTrackNotFound {
            lib_root: lib_root.to_owned(),
            track_path: target.to_owned(),
        }
        .into());
    }

    delete_track_checked(&abs_path)
}

/// パス文字列を指定してライブラリから削除
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - target: 削除対象のライブラリ内パス
///
/// # Errors
/// - alk_base_2_domain::Error::PathStrNotFound: 指定されたパスが見つからなかった場合
pub fn delete_path_str(lib_root: &Path, target: &NonEmptyString) -> Result<()> {
    let target_abs = lib_root.join(target);

    //ファイルが存在しない
    if !target_abs.exists() {
        Err(DomainError::FilePathStrNotFound {
            lib_root: lib_root.to_owned(),
            path_str: target.to_owned(),
        }
        .into())
    }
    // フォルダ
    else if target_abs.is_dir() {
        fs::remove_dir_all(&target_abs)?;
        Ok(())
    }
    // ファイル
    else {
        delete_track_checked(&target_abs)
    }
}

/// ライブラリから曲を削除(曲ファイルが存在することの確認後)
///
/// # Arguments
/// - abs_path: 削除対象曲ファイルの絶対パス
fn delete_track_checked(abs_path: &Path) -> Result<()> {
    fs::remove_file(abs_path).map_err(|e| DomainError::FileIoError(abs_path.to_owned(), e))?;

    let lrc_path = utils::get_lrc_path(abs_path);

    //歌詞ファイルもあれば削除
    if lrc_path.exists() {
        fs::remove_file(&lrc_path).map_err(|e| DomainError::FileIoError(lrc_path, e))?;
    }

    Ok(())
}

/// ライブラリから曲をゴミ箱に移動
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - target: 削除対象の曲のライブラリ内パス
pub fn trash_track(lib_root: &Path, target: &LibraryTrackPath) -> Result<()> {
    //ファイルが存在するか確認
    let abs_path = target.abs(lib_root);
    if !abs_path.exists() {
        return Err(DomainError::FileTrackNotFound {
            lib_root: lib_root.to_owned(),
            track_path: target.to_owned(),
        }
        .into());
    }

    trash_track_checked(&abs_path)
}

/// パス文字列を指定してライブラリからゴミ箱に移動
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - target: 削除対象のライブラリ内パス
///
/// # Errors
/// - alk_base_2_domain::Error::PathStrNotFound: 指定されたパスが見つからなかった場合
pub fn trash_path_str(lib_root: &Path, target: &NonEmptyString) -> Result<()> {
    let target_abs = lib_root.join(target);

    //ファイルが存在しない
    if !target_abs.exists() {
        Err(DomainError::FilePathStrNotFound {
            lib_root: lib_root.to_owned(),
            path_str: target.to_owned(),
        }
        .into())
    }
    // フォルダ
    else if target_abs.is_dir() {
        trash::delete(&target_abs)?;
        Ok(())
    }
    // ファイル
    else {
        trash_track_checked(&target_abs)
    }
}

/// ライブラリから曲をゴミ箱に移動(曲ファイルが存在することの確認後)
///
/// # Arguments
/// - abs_path: 削除対象曲ファイルの絶対パス
fn trash_track_checked(abs_path: &Path) -> Result<()> {
    trash::delete(abs_path)?;

    let lrc_path = utils::get_lrc_path(abs_path);

    //歌詞ファイルもあれば削除
    if lrc_path.exists() {
        trash::delete(&lrc_path)?;
    }

    Ok(())
}
