use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use crate::db::DbTransaction;

/// 曲とプレイリストの紐づけ関係のDBリポジトリ
#[async_trait]
pub trait DbPlaylistTrackRepository {
    //曲を全プレイリストから削除
    async fn delete_track_from_all_playlists<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        track_id: i32,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbPlaylistTrackRepository {
    pub inner: MockDbPlaylistTrackRepositoryInner,
}
#[async_trait]
impl DbPlaylistTrackRepository for MockDbPlaylistTrackRepository {
    async fn delete_track_from_all_playlists<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        track_id: i32,
    ) -> Result<()> {
        self.inner.delete_track_from_all_playlists(track_id)
    }
}
mock! {
    pub DbPlaylistTrackRepositoryInner {
        pub fn delete_track_from_all_playlists(
            &self,
            track_id: i32,
        ) -> Result<()>;
    }
}
