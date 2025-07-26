//! checkコマンド関連

mod command_check;
pub use command_check::CommandCheck;

pub mod args;
pub use args::CommandCheckArgs;

mod resolve_existance_result;
use resolve_existance_result::ResolveFileExistanceResult;

mod domain;
mod messages;

mod resolve_dap;
#[cfg(test)]
use resolve_dap::MockResolveDap;
pub use resolve_dap::{ResolveDap, ResolveDapImpl};
mod resolve_data_match;
#[cfg(test)]
use resolve_data_match::MockResolveDataMatch;
pub use resolve_data_match::{ResolveDataMatch, ResolveDataMatchImpl};
mod resolve_existance;
#[cfg(test)]
use resolve_existance::MockResolveExistance;
pub use resolve_existance::{ResolveExistance, ResolveExistanceImpl};

mod track_item_conflict;
use track_item_conflict::TrackItemConflict;
