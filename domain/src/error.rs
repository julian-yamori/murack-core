use crate::path::{LibDirPath, LibPathStr, LibTrackPath};
use std::path::PathBuf;

/// murack-core domain層のエラー
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// ファイルIO汎用エラー
    #[error("{1}: {0}")]
    FileIoError(PathBuf, std::io::Error),

    #[error("曲ファイルが存在しません: {track_path} (in {lib_root})")]
    FileTrackNotFound {
        lib_root: PathBuf,
        track_path: LibTrackPath,
    },
    #[error("指定されたパスが存在しません: {path_str} (in {lib_root})")]
    FilePathStrNotFound {
        lib_root: PathBuf,
        path_str: LibPathStr,
    },
    #[error("曲ファイルが既に存在しています: {track_path} (in {lib_root})")]
    FileTrackAlreadyExists {
        lib_root: PathBuf,
        track_path: LibTrackPath,
    },
    #[error("指定されたパスが既に存在しています: {path_str} (in {lib_root})")]
    FilePathStrAlreadyExists {
        lib_root: PathBuf,
        path_str: LibPathStr,
    },

    #[error("曲データがDBに存在しません: {0}")]
    DbTrackNotFound(LibTrackPath),
    #[error("DBに指定されたパスが存在しません: {0}")]
    DbPathStrNotFound(LibPathStr),
    #[error("曲データが既にDBに存在します: {0}")]
    DbTrackAlreadyExists(LibTrackPath),
    #[error("フォルダがDBに存在しません: {0}")]
    DbFolderPathNotFound(LibDirPath),
    #[error("フォルダIDがDBに存在しません: {0}")]
    DbFolderIdNotFound(i32),
    #[error("フォルダが既にDBに存在します: {0}")]
    DbFolderAlreadyExists(LibDirPath),

    #[error("相対パスの取得に失敗しました: \"{parent}\" 内の \"{track}\"")]
    GetRelativePathFailed {
        track: LibTrackPath,
        parent: LibDirPath,
    },
}
