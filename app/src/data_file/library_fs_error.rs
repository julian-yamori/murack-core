use std::path::PathBuf;

use murack_core_domain::{NonEmptyString, path::LibraryTrackPath};

/// ライブラリの曲ファイル操作時のエラー
#[derive(thiserror::Error, Debug)]
pub enum LibraryFsError {
    #[error("曲ファイルが存在しません: {track_path} (in {lib_root})")]
    FileTrackNotFound {
        lib_root: PathBuf,
        track_path: LibraryTrackPath,
    },
    #[error("指定されたパスが存在しません: {path_str} (in {lib_root})")]
    FilePathStrNotFound {
        lib_root: PathBuf,
        path_str: NonEmptyString,
    },
    #[error("曲ファイルが既に存在しています: {track_path} (in {lib_root})")]
    FileTrackAlreadyExists {
        lib_root: PathBuf,
        track_path: LibraryTrackPath,
    },
    #[error("指定されたパスが既に存在しています: {path_str} (in {lib_root})")]
    FilePathStrAlreadyExists {
        lib_root: PathBuf,
        path_str: NonEmptyString,
    },
}
