use crate::path::LibraryDirectoryPath;

/// murack-core domain層のエラー
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("フォルダがDBに存在しません: {0}")]
    DbFolderPathNotFound(LibraryDirectoryPath),
    #[error("フォルダIDがDBに存在しません: {0}")]
    DbFolderIdNotFound(i32),
}
