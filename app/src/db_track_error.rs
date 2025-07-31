use murack_core_domain::{NonEmptyString, path::LibraryTrackPath};

/// DB の曲関連のエラー
#[derive(thiserror::Error, Debug)]
pub enum DbTrackError {
    #[error("曲データがDBに存在しません: {0}")]
    DbTrackNotFound(LibraryTrackPath),
    #[error("DBに指定されたパスが存在しません: {0}")]
    DbPathStrNotFound(NonEmptyString),
    #[error("曲データが既にDBに存在します: {0}")]
    DbTrackAlreadyExists(LibraryTrackPath),
}
