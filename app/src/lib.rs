//! application層
//!
//! domainを操作する、CUIアプリケーションのロジック

mod error;
pub use error::Error;

#[macro_use]
pub mod cui;

mod config;
pub use config::Config;

pub mod command;
pub mod data_file;
pub mod db_common;

#[cfg(test)]
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
