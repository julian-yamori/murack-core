//! サブコマンド毎の機能定義

mod add;
pub use add::CommandAdd;

mod aw_get;
pub use aw_get::CommandArtworkGet;

mod check;
pub use check::CommandCheck;

mod cmd_move;
pub use cmd_move::CommandMove;

mod help;
pub use help::CommandHelp;

mod playlist;
pub use playlist::CommandPlaylist;

mod remove;
pub use remove::CommandRemove;

//todo 実装が怪しいのと、利用機会が遠そうなので無効化
// mod replace;
// pub use replace::CommandReplace;
