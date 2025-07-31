//! アートワーク (曲に追加する画像データ) 関連の機能

pub mod artwork_error;
pub use artwork_error::ArtworkError;

pub mod artwork_repository;

pub mod picture;
pub use picture::Picture;
