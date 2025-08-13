//! オーディオメタデータの読み書き機能

pub mod file_io;

mod file_mid_metadata;
use file_mid_metadata::FileMidMetadata;

mod format_type;
pub use format_type::FormatType;

pub mod formats;

mod audio_metadata_model;
pub use audio_metadata_model::TrackArtwork;
