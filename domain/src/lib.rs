//! domain層

mod error;
pub use error::Error;

pub mod db_components;

pub mod artwork;
pub mod db_utils;
pub mod filter;
pub mod folder;

mod non_empty_string;
pub use non_empty_string::{EmptyStringError, NonEmptyString};

pub mod path;
pub mod playlist;
pub mod string_order_cnv;
pub mod sync;
pub mod tag;
pub mod track;
pub mod track_query;

pub mod test_utils;

#[macro_use]
extern crate derive_new;

#[cfg(test)]
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
