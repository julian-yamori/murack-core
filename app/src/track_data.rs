//! 曲データ (特に `AudioMetadata` の管理)

pub mod file_io;

mod file_mid_metadata;
use file_mid_metadata::FileMidMetadata;

mod format_type;
pub use format_type::FormatType;

pub mod formats;

pub mod audio_metadata;
pub use audio_metadata::{AudioMetadata, TrackArtwork};
