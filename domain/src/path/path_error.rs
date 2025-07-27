use std::ffi::OsString;

use thiserror::Error;

use crate::EmptyStringError;

#[derive(Error, Debug)]
pub enum PathError {
    #[error(transparent)]
    EmptyString(#[from] EmptyStringError),

    #[error("failed to decode path to UTF-8 : {}", .from.display())]
    FailedToDecode { from: OsString },
}
