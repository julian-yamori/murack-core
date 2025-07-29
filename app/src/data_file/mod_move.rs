//! ファイルの移動機能

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use murack_core_domain::{Error as DomainError, NonEmptyString};

use crate::data_file::utils;

/// パス文字列を指定してライブラリ内のファイル/フォルダを移動
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - src: 移動元のライブラリ内パス
/// - dest: 移動先のライブラリ内パス
pub fn move_path_str(lib_root: &Path, src: &NonEmptyString, dest: &NonEmptyString) -> Result<()> {
    let src_abs = lib_root.join(src);
    if !src_abs.exists() {
        return Err(DomainError::FilePathStrNotFound {
            lib_root: lib_root.to_owned(),
            path_str: src.to_owned(),
        }
        .into());
    }

    let dest_abs = lib_root.join(dest);
    if dest_abs.exists() {
        return Err(DomainError::FilePathStrAlreadyExists {
            lib_root: lib_root.to_owned(),
            path_str: dest.to_owned(),
        }
        .into());
    }

    //移動先で不足しているディレクトリを作成
    if let Some(parent) = dest_abs.parent() {
        fs::create_dir_all(parent).map_err(|e| DomainError::FileIoError(parent.to_owned(), e))?;
    }

    //移動を実行
    fn_move(&src_abs, &dest_abs)?;

    //扱ったものがファイルで、歌詞ファイルがあれば移動
    if dest_abs.is_file() {
        let src_lrc = utils::get_lrc_path(&src_abs);
        if src_lrc.exists() {
            let dest_lrc = utils::get_lrc_path(&dest_abs);
            fn_move(&src_lrc, &dest_lrc)?;
        }
    }

    Ok(())
}

fn fn_move(from: &Path, to: &Path) -> Result<()> {
    fs::rename(from, to).with_context(|| {
        format!(
            "failed to move file. from: {} to: {}",
            from.display(),
            to.display(),
        )
    })
}
