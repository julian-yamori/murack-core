//! ファイルシステムのライブラリフォルダ内の曲データ取扱
//! (旧 murack-core-data-file クレート)

mod copy;
pub use copy::{copy_track_over_lib, overwrite_track_over_lib};

mod delete;
pub use delete::{delete_path_str, delete_track, trash_path_str, trash_track};

pub mod library_fs_error;
pub use library_fs_error::LibraryFsError;

mod mod_move;
pub use mod_move::move_path_str;

mod search;
pub use search::{search_all, search_by_lib_path, search_track_outside_lib};

mod track_sync;
pub use track_sync::{overwrite_track_sync, read_metadata, read_track_sync};

mod utils;

/// パス文字列で指定されたパスが存在するか確認
///
/// # Arguments
/// - lib_root: ライブラリルートの絶対パス
/// - path_str: 確認対象のライブラリ内パス
///
/// # Returns
/// ファイル、もしくはフォルダが存在する場合true、どちらもなければfalse
pub fn is_exist_path_str(
    lib_root: &std::path::Path,
    path_str: &murack_core_domain::NonEmptyString,
) -> anyhow::Result<bool> {
    let abs_path = lib_root.join(path_str);
    Ok(abs_path.exists())
}
