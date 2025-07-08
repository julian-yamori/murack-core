//! application層
//!
//! domainを操作する、CUIアプリケーションのロジック

mod error;
pub use error::Error;

mod app_components;
pub use app_components::AppComponents;

#[macro_use]
pub mod cui;

mod config;
pub use config::Config;

pub mod command;
