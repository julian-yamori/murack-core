use std::path::PathBuf;

/// murack-core メディアデータのエラー
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// ファイルIO汎用エラー
    #[error("{0}: {1}")]
    FileIoError(PathBuf, std::io::Error),

    /// 非対応の音声フォーマット
    #[error("非対応の音声フォーマットです: {fmt}")]
    UnsupportedAudioFormat { fmt: String },
    /// 非対応のアートワークフォーマット
    #[error("非対応のアートワーク形式です: {fmt}")]
    UnsupportedArtworkFmt { fmt: String },
    /// リリース日が不正値
    #[error("リリース日の値が不正です: {value_info}")]
    InvalidReleaseDate { value_info: String },
    /// 再生時間が不正値
    #[error("再生時間の値が不正です: {msg}")]
    InvalidDuration { msg: String },

    /// FLAC: streaminfoブロックがない
    #[error("StreamInfoブロックがありません")]
    FlacStreamInfoNone,
    /// FLAC: VorbisCommentブロックがない
    #[error("VorbisCommentブロックがありません")]
    FlacVorbisCommentNone,
    /// FLAC: 文字列値から数値への変換エラー
    #[error("VorbisCommentの値が数値に変換できませんでした: {key}={value}")]
    FlacIntegerPaseError {
        /// 変換に失敗した値のVorbisComment key
        key: String,
        /// 変換しようとした文字列
        value: String,
    },

    /// 非対応のID3バージョン
    #[error("非対応のID3バージョンです: {}", display_id3_version(.0))]
    UnsupportedId3Version(id3::Version),
    #[error("アートワークのPicture typeが重複しています: {type_num}")]
    Id3PictureTypeDuplicated { type_num: u8 },

    #[error("m4aでは{field}に0を設定できません。")]
    M4ANumberZero {
        /// 0を設定しようとした項目名
        field: String,
    },
}

fn display_id3_version(version: &id3::Version) -> &'static str {
    match version {
        id3::Version::Id3v22 => "v2.2",
        id3::Version::Id3v23 => "v2.3",
        id3::Version::Id3v24 => "v2.4",
    }
}
