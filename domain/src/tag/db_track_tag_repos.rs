use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;
use sqlx::PgTransaction;

/// 曲とタグの紐づけ関係のリポジトリ
#[async_trait]
pub trait DbTrackTagRepository {
    /// 曲から全てのタグを削除
    async fn delete_all_tags_from_track<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track_id: i32,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbTrackTagRepository {
    pub inner: MockDbTrackTagRepositoryInner,
}
#[async_trait]
impl DbTrackTagRepository for MockDbTrackTagRepository {
    async fn delete_all_tags_from_track<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        track_id: i32,
    ) -> Result<()> {
        self.inner.delete_all_tags_from_track(track_id)
    }
}
mock! {
    pub DbTrackTagRepositoryInner {
        pub fn delete_all_tags_from_track(
            &self,
            track_id: i32,
        ) -> Result<()>;
    }
}
