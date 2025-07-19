use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use crate::db::DbTransaction;

/// 曲とタグの紐づけ関係のリポジトリ
#[async_trait]
pub trait DbSongTagRepository {
    /// 曲から全てのタグを削除
    async fn delete_all_tags_from_song<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbSongTagRepository {
    pub inner: MockDbSongTagRepositoryInner,
}
#[async_trait]
impl DbSongTagRepository for MockDbSongTagRepository {
    async fn delete_all_tags_from_song<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()> {
        self.inner.delete_all_tags_from_song(song_id)
    }
}
mock! {
    pub DbSongTagRepositoryInner {
        pub fn delete_all_tags_from_song(
            &self,
            song_id: i32,
        ) -> Result<()>;
    }
}
