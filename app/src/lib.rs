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

/// DBコネクションプールに接続
pub async fn db_pool_connect(database_url: &str) -> anyhow::Result<sqlx::PgPool> {
    Ok(sqlx::postgres::PgPoolOptions::new()
        .connect(database_url)
        .await?)
}
