use std::path::Path;

use anyhow::{Context, Result};

use crate::audio_metadata::AudioMetaDataError;

/// オーディオファイルのフォーマット種別
pub enum FormatType {
    Flac,
    Mp3,
    M4a,
}

impl FormatType {
    /// パスからフォーマット種別を判別
    pub fn from_path(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .with_context(|| format!("拡張子の取得に失敗しました: {}", path.display()))?
            .to_str()
            .with_context(|| format!("UTF-8への変換に失敗しました: {}", path.display()))?;

        Self::from_ext(ext)
    }

    /// 拡張子からフォーマット種別を判別
    pub fn from_ext(ext: &str) -> Result<Self> {
        match ext {
            "mp3" => Ok(FormatType::Mp3),
            "flac" => Ok(FormatType::Flac),
            //"ogg" | "oga" => FormatType::Ogg,
            "m4a" | "m4b" | "m4p" | "m4v" | "isom" | "mp4" => Ok(FormatType::M4a),
            ext => Err(AudioMetaDataError::UnsupportedAudioFormat {
                fmt: ext.to_owned(),
            }
            .into()),
        }
    }
}
