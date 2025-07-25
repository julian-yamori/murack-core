use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use super::TrackArtwork;
use crate::db::DbTransaction;

/// アートワーク関係のDBリポジトリ
#[async_trait]
pub trait DbArtworkRepository {
    /// 曲に紐づくアートワークの情報を取得する
    /// # Arguments
    /// - track_id: アートワーク情報を取得する曲のID
    /// # Returns
    /// 指定された曲に紐づく全アートワークの情報
    async fn get_track_artworks<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        track_id: i32,
    ) -> Result<Vec<TrackArtwork>>;

    /// 曲にアートワークの紐付きを登録
    ///
    /// orderは無視し、関数内で上書きする。
    ///
    /// # Arguments
    /// - track_id: 紐付けを登録する曲のID
    /// - track_artworks: 曲に紐づく全てのアートワークの情報
    async fn register_track_artworks<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        track_id: i32,
        track_artworks: &[TrackArtwork],
    ) -> Result<()>;

    /// 曲へのアートワーク紐付き情報を削除
    ///
    /// どの曲にも紐付かないアートワークは、DBから削除する
    ///
    /// # Arguments
    /// - track_id 紐付けを削除する曲のID
    async fn unregister_track_artworks<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        track_id: i32,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbArtworkRepository {
    pub inner: MockDbArtworkRepositoryInner,
}
#[async_trait]
impl DbArtworkRepository for MockDbArtworkRepository {
    async fn get_track_artworks<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        track_id: i32,
    ) -> Result<Vec<TrackArtwork>> {
        self.inner.get_track_artworks(track_id)
    }

    async fn register_track_artworks<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        track_id: i32,
        track_artworks: &[TrackArtwork],
    ) -> Result<()> {
        self.inner.register_track_artworks(track_id, track_artworks)
    }

    async fn unregister_track_artworks<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        track_id: i32,
    ) -> Result<()> {
        self.inner.unregister_track_artworks(track_id)
    }
}
mock! {
    pub DbArtworkRepositoryInner{
        pub fn get_track_artworks(
            &self,
            track_id: i32,
        ) -> Result<Vec<TrackArtwork>>;

        pub fn register_track_artworks(
            &self,
            track_id: i32,
            track_artworks: &[TrackArtwork],
        ) -> Result<()>;

        pub fn unregister_track_artworks(&self, track_id: i32)
            -> Result<()>;
    }
}
