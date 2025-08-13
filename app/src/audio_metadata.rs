//! オーディオメタデータの読み書き機能

mod format_type;
pub use format_type::FormatType;

pub mod formats;

mod audio_metadata_model;
pub use audio_metadata_model::{AudioMetaData, TrackArtwork};
