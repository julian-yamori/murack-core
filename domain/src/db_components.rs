use crate::track::DbTrackRepositoryImpl;

/// data層DB機能のDIを解決するオブジェクト
pub struct DbComponents {}

impl DbComponents {
    pub fn new() -> Self {
        Self {}
    }

    pub fn db_track_repository(&self) -> TypeDbTrackRepository {
        DbTrackRepositoryImpl::new()
    }
}

impl Default for DbComponents {
    fn default() -> Self {
        Self::new()
    }
}

pub type TypeDbTrackRepository = DbTrackRepositoryImpl;
