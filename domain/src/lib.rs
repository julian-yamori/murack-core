//! domain層

mod error;
pub use error::Error;

pub mod artwork;
pub mod check;
pub mod dap;
pub mod db_wrapper;
pub mod filter;
pub mod folder;
pub mod path;
pub mod playlist;
pub mod song;
pub mod string_order_cnv;
pub mod sync;
pub mod tag;

mod file_lib_repos;
pub use file_lib_repos::{FileLibraryRepository, MockFileLibraryRepository};

pub mod test_utils;

#[macro_use]
extern crate derive_new;
