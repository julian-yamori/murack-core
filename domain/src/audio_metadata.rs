//! オーディオメタデータの読み書き機能

mod format_type;
pub use format_type::FormatType;

pub mod formats;

pub mod audio_metadata_error;
pub use audio_metadata_error::AudioMetaDataError;

mod audio_metadata_model;
pub use audio_metadata_model::{AudioMetaData, AudioMetaDataEntry};

mod audio_picture;
pub use audio_picture::{AudioPicture, AudioPictureEntry};
