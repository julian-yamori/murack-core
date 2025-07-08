use std::path::PathBuf;

/// WalkBase2 data_file層のエラー
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ファイルが存在しません: {0})")]
    AbsFileNotFound(PathBuf),
}
