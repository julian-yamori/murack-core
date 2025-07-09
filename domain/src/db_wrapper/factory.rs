use super::ConnectionWrapper;
use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

pub enum ConnectionFactory {
    File(PathBuf),
    Memory,
    Dummy,
}

impl ConnectionFactory {
    pub fn open(&self) -> Result<ConnectionWrapper> {
        Ok(match self {
            Self::File(path) => ConnectionWrapper::Real(Connection::open(path)?),
            Self::Memory => ConnectionWrapper::Real(Connection::open_in_memory()?),
            Self::Dummy => ConnectionWrapper::Dummy { commited: false },
        })
    }
}
