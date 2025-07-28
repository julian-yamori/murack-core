//! SQLite DBでdomain modelを読み込み・保存

pub mod db_components;

pub mod folder;
pub mod tag;
pub mod track;

#[macro_use]
extern crate derive_new;

#[cfg(test)]
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
