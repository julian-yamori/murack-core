use crate::path::{LibraryDirectoryPath, LibraryTrackPath};

/// murack-core domain層のエラー
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("フォルダがDBに存在しません: {0}")]
    DbFolderPathNotFound(LibraryDirectoryPath),
    #[error("フォルダIDがDBに存在しません: {0}")]
    DbFolderIdNotFound(i32),
    #[error("フォルダが既にDBに存在します: {0}")]
    DbFolderAlreadyExists(LibraryDirectoryPath),

    #[error("相対パスの取得に失敗しました: \"{parent}\" 内の \"{track}\"")]
    GetRelativePathFailed {
        track: LibraryTrackPath,
        parent: LibraryDirectoryPath,
    },
}
