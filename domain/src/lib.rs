//! domain層

pub mod artwork;

pub mod audio_metadata;

pub mod db_utils;
pub mod filter;
pub mod folder;

mod non_empty_string;
pub use non_empty_string::{EmptyStringError, NonEmptyString};

pub mod path;
pub mod playlist;
pub mod string_order_cnv;
pub mod track;
pub mod track_query;

pub mod test_utils;

#[cfg(test)]
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
