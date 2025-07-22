//! サブコマンド毎の機能定義

pub mod add;
pub use add::{CommandAdd, CommandAddArgs};

pub mod aw_get;
pub use aw_get::{CommandArtworkGet, CommandArtworkGetArgs};

pub mod check;
pub use check::{
    CommandCheck, CommandCheckArgs, ResolveDapImpl, ResolveDataMatchImpl, ResolveExistanceImpl,
};

pub mod cmd_move;
pub use cmd_move::{CommandMove, CommandMoveArgs};

pub mod help;
pub use help::CommandHelp;

pub mod playlist;
pub use playlist::CommandPlaylist;

pub mod remove;
pub use remove::{CommandRemove, CommandRemoveArgs};

//todo 実装が怪しいのと、利用機会が遠そうなので無効化
// pub mod replace;
// pub use replace::CommandReplace;
