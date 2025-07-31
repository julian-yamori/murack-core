use std::path::PathBuf;

/// audio metadata 関連のエラー
#[derive(thiserror::Error, Debug)]
pub enum AudioMetaDataError {
    /// ファイルIO汎用エラー
    #[error("{0}: {1}")]
    FileIoError(PathBuf, std::io::Error),

    /// 非対応の音声フォーマット
    #[error("非対応の音声フォーマットです: {fmt}")]
    UnsupportedAudioFormat { fmt: String },
}
