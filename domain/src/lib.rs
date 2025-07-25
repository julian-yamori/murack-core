//! domain層

mod error;
pub use error::Error;

pub mod artwork;
pub mod check;
pub mod dap;
pub mod filter;
pub mod folder;
pub mod path;
pub mod playlist;
pub mod string_order_cnv;
pub mod sync;
pub mod tag;
pub mod track;

mod file_lib_repos;
pub use file_lib_repos::{FileLibraryRepository, MockFileLibraryRepository};

pub mod db;

pub mod test_utils;

#[macro_use]
extern crate derive_new;
