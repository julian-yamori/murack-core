//! application層
//!
//! Murack Sync に以降予定のコード群

pub mod app_artwork_repository;

pub mod audio_metadata;

#[macro_use]
pub mod cui;

mod config;
pub use config::Config;

pub mod command;
pub mod data_file;
pub mod db_common;

pub mod db_track_error;
pub use db_track_error::DbTrackError;

pub mod track_sync;

#[cfg(test)]
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
