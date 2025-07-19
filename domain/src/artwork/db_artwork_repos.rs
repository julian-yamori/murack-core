use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use super::SongArtwork;
use crate::db::DbTransaction;

/// アートワーク関係のDBリポジトリ
#[async_trait]
pub trait DbArtworkRepository {
    /// 曲に紐づくアートワークの情報を取得する
    /// # Arguments
    /// - song_id: アートワーク情報を取得する曲のID
    /// # Returns
    /// 指定された曲に紐づく全アートワークの情報
    async fn get_song_artworks<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<Vec<SongArtwork>>;

    /// 曲にアートワークの紐付きを登録
    ///
    /// orderは無視し、関数内で上書きする。
    ///
    /// # Arguments
    /// - song_id: 紐付けを登録する曲のID
    /// - song_artworks: 曲に紐づく全てのアートワークの情報
    async fn register_song_artworks<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        song_artworks: &[SongArtwork],
    ) -> Result<()>;

    /// 曲へのアートワーク紐付き情報を削除
    ///
    /// どの曲にも紐付かないアートワークは、DBから削除する
    ///
    /// # Arguments
    /// - song_id 紐付けを削除する曲のID
    async fn unregister_song_artworks<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbArtworkRepository {
    pub inner: MockDbArtworkRepositoryInner,
}
#[async_trait]
impl DbArtworkRepository for MockDbArtworkRepository {
    async fn get_song_artworks<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<Vec<SongArtwork>> {
        self.inner.get_song_artworks(song_id)
    }

    async fn register_song_artworks<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        song_id: i32,
        song_artworks: &[SongArtwork],
    ) -> Result<()> {
        self.inner.register_song_artworks(song_id, song_artworks)
    }

    async fn unregister_song_artworks<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()> {
        self.inner.unregister_song_artworks(song_id)
    }
}
mock! {
    pub DbArtworkRepositoryInner{
        pub fn get_song_artworks(
            &self,
            song_id: i32,
        ) -> Result<Vec<SongArtwork>>;

        pub fn register_song_artworks(
            &self,
            song_id: i32,
            song_artworks: &[SongArtwork],
        ) -> Result<()>;

        pub fn unregister_song_artworks(&self, song_id: i32)
            -> Result<()>;
    }
}
