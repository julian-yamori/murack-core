//! infra層
//! 全体の依存関係を整備し、appに繋ぎこむ

mod run;
pub use run::run;

mod app_args;
use app_args::AppArgs;

mod registry;
use registry::Registry;
