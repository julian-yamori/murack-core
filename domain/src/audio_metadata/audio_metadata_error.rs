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
    /// 非対応のアートワークフォーマット
    #[error("非対応のアートワーク形式です: {fmt}")]
    UnsupportedArtworkFmt { fmt: String },
}
