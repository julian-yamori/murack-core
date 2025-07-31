use crate::path::LibraryDirectoryPath;

/// folder_paths テーブル関連のエラー
#[derive(thiserror::Error, Debug)]
pub enum FolderPathError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("フォルダがDBに存在しません: {0}")]
    DbFolderPathNotFound(LibraryDirectoryPath),

    #[error("フォルダIDがDBに存在しません: {0}")]
    DbFolderIdNotFound(i32),
}
