//! SQLite DBでdomain modelを読み込み・保存

mod error;
pub use error::Error;

pub mod db_components;

pub mod artwork;
pub mod folder;
pub mod playlist;
pub mod tag;
pub mod track;
pub mod track_lister;

mod like_esc;

#[macro_use]
extern crate derive_new;

#[cfg(test)]
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
