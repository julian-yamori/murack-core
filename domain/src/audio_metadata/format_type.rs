use std::path::{Path, PathBuf};

/// オーディオファイルのフォーマット種別
pub enum FormatType {
    Flac,
    Mp3,
    M4a,
}

impl FormatType {
    /// パスからフォーマット種別を判別
    pub fn from_path(path: &Path) -> Result<Self, FormatTypeError> {
        let ext = path
            .extension()
            .ok_or_else(|| FormatTypeError::FailedToGetExtension(path.to_owned()))?
            .to_str()
            .ok_or_else(|| FormatTypeError::FailedToEncodeUtf8(path.to_owned()))?;

        Self::from_ext(ext)
    }

    /// 拡張子からフォーマット種別を判別
    pub fn from_ext(ext: &str) -> Result<Self, FormatTypeError> {
        match ext {
            "mp3" => Ok(FormatType::Mp3),
            "flac" => Ok(FormatType::Flac),
            //"ogg" | "oga" => FormatType::Ogg,
            "m4a" | "m4b" | "m4p" | "m4v" | "isom" | "mp4" => Ok(FormatType::M4a),

            ext => Err(FormatTypeError::UnsupportedExtension {
                extention: ext.to_owned(),
            }),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FormatTypeError {
    #[error("拡張子の取得に失敗しました: {}", .0.display())]
    FailedToGetExtension(PathBuf),

    #[error("UTF-8への変換に失敗しました: {}", .0.display())]
    FailedToEncodeUtf8(PathBuf),

    #[error("非対応の音声ファイルの拡張子です: {extention}")]
    UnsupportedExtension { extention: String },
}
