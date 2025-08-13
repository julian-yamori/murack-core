//! アートワーク (曲に追加する画像データ) 関連の機能

pub mod artwork_error;
pub use artwork_error::ArtworkError;

pub mod artwork_hash;
pub use artwork_hash::ArtworkHash;

pub mod artwork_repository;

pub mod mini_image;
pub use mini_image::MiniImage;

mod track_artwork;
pub use track_artwork::TrackArtwork;
