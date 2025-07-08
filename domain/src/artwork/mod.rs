//! アートワーク関係の機能

mod song_artwork;
pub use song_artwork::SongArtwork;

mod db_artwork_repos;
pub use db_artwork_repos::{DbArtworkRepository, MockDbArtworkRepository};
