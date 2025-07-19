use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use crate::db::DbTransaction;

/// 曲とプレイリストの紐づけ関係のDBリポジトリ
#[async_trait]
pub trait DbPlaylistSongRepository {
    //曲を全プレイリストから削除
    async fn delete_song_from_all_playlists<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbPlaylistSongRepository {
    pub inner: MockDbPlaylistSongRepositoryInner,
}
#[async_trait]
impl DbPlaylistSongRepository for MockDbPlaylistSongRepository {
    async fn delete_song_from_all_playlists<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()> {
        self.inner.delete_song_from_all_playlists(song_id)
    }
}
mock! {
    pub DbPlaylistSongRepositoryInner {
        pub fn delete_song_from_all_playlists(
            &self,
            song_id: i32,
        ) -> Result<()>;
    }
}
