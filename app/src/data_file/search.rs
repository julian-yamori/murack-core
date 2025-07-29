//! 曲パスの検索機能

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use murack_core_domain::{Error as DomainError, NonEmptyString, path::LibraryTrackPath};

use crate::data_file::utils;

/// ライブラリのフォルダ内の全ての曲のパスを列挙
pub fn search_all(lib_root: &Path) -> Result<Vec<LibraryTrackPath>> {
    search_lib_path_from(lib_root, lib_root)
}

/// ライブラリのフォルダ内で、指定パスに該当する曲のパスを列挙
///
/// 指定されたパスがファイルなら、そのファイルパスを返す。
/// 指定されたパスがディレクトリなら、そのディレクトリに含まれる全音声ファイルのパスを返す。(子ディレクトリ内も含む)
///
/// 指定パスにファイル/ディレクトリがない、
/// もしくは指定ディレクトリ内に音声ファイルがない場合、
/// 長さ0のVecを返す。
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - target: 検索対象のライブラリ内パス
pub fn search_by_lib_path(
    lib_root: &Path,
    target: &NonEmptyString,
) -> Result<Vec<LibraryTrackPath>> {
    let target_abs = lib_root.join(target);

    search_lib_path_from(lib_root, &target_abs)
}

/// 絶対パスを指定して、その配下の曲のパスを `Vec<LibraryTrackPath>` で返す
///
/// search_all() と search_by_lib_path() の共通部分
fn search_lib_path_from(lib_root: &Path, target_abs: &Path) -> Result<Vec<LibraryTrackPath>> {
    search_track_abs_path(target_abs)?
        .into_iter()
        .map(|abs_path| {
            //絶対パス->ライブラリパスに変換
            let rel_path = abs_path.strip_prefix(lib_root).unwrap();
            let track_path = LibraryTrackPath::try_from(rel_path.to_owned())?;
            Ok(track_path)
        })
        .collect()
}

/// ライブラリ外で、指定パスに該当する曲のパスを列挙
///
/// 指定されたパスがファイルなら、そのファイルパスを返す。
/// 指定されたパスがディレクトリなら、そのディレクトリに含まれる全音声ファイルのパスを返す。(子ディレクトリ内も含む)
///
/// 指定パスにファイル/ディレクトリがない、
/// もしくは指定ディレクトリ内に音声ファイルがない場合、
/// 長さ0のVecを返す。
///
/// # Arguments
/// - target: 検索する絶対パス
pub fn search_track_outside_lib(target: &Path) -> Result<Vec<PathBuf>> {
    search_track_abs_path(target)
}

/// 絶対パスを指定し、該当する曲の絶対パスを列挙
fn search_track_abs_path(target: &Path) -> Result<Vec<PathBuf>> {
    //ファイルが存在しない
    if !target.exists() {
        Ok(vec![])
    }
    //フォルダなら再帰実行して返す
    else if target.is_dir() {
        let mut abs_list = vec![];
        search_dir_rec(&mut abs_list, target)?;

        Ok(abs_list)
    }
    //ファイル
    else {
        Ok(vec![target.to_owned()])
    }
}

/// ディレクトリパスを指定して音声ファイルを列挙(再帰実行)
///
/// # Arguments
/// - dest: 列挙されたファイル絶対パスの格納先リスト
/// - path: 検索対象ディレクトリの絶対パス
fn search_dir_rec(dest: &mut Vec<PathBuf>, path: &Path) -> Result<()> {
    //子要素の走査
    let entries = match fs::read_dir(path) {
        Ok(i) => i.collect::<Vec<std::result::Result<std::fs::DirEntry, std::io::Error>>>(),
        Err(e) => return Err(DomainError::FileIoError(path.to_owned(), e).into()),
    };

    //直下ディレクトリの絶対パスリスト
    //直下ファイルをdestにまとめて格納した後に、子ディレクトリを処理。
    let mut child_dirs = Vec::<PathBuf>::new();

    //ディレクトリ内のファイルを列挙
    for entry in entries {
        let entry =
            entry.with_context(|| format!("failed to get file entry in: {}", path.display()))?;

        let entry_path = entry.path();

        let file_type = match entry.metadata() {
            Ok(m) => m.file_type(),
            Err(e) => {
                return Err(DomainError::FileIoError(entry_path, e).into());
            }
        };

        //ディレクトリなら、一旦パスリストに追加
        if file_type.is_dir() {
            child_dirs.push(entry_path);
            continue;
        }
        //音声ファイルの拡張子なら、絶対パスを追加
        else if utils::is_audio_ext(&entry_path) {
            dest.push(entry_path);
        }
    }

    //子ディレクトリについて処理
    for child_path in child_dirs.iter() {
        search_dir_rec(dest, child_path)?;
    }

    Ok(())
}
