//! ファイルシステムのライブラリフォルダ内の曲データ取扱

mod error;
pub use error::Error;

mod file_lib_repos_impl;
pub use file_lib_repos_impl::FileLibraryRepositoryImpl;

mod dap_repos_impl;
pub use dap_repos_impl::DapRepositoryImpl;

mod copy;
mod delete;
mod mod_move;
mod search;
mod track_sync;

mod utils;
