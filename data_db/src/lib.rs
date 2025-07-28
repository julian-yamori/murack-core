//! SQLite DBでdomain modelを読み込み・保存

#[cfg(test)]
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
