//! SQLite DBでdomain modelを読み込み・保存

pub mod db_components;

#[cfg(test)]
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
