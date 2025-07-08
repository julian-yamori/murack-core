//! オーディオメタデータの読み書き機能

mod format_type;
pub use format_type::FormatType;

pub mod formats;

mod audio_metadata;
pub use audio_metadata::{AudioMetaData, AudioMetaDataEntry};
mod audio_picture;
pub use audio_picture::{AudioPicture, AudioPictureEntry};
