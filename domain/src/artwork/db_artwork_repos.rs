use super::SongArtwork;
use crate::db_wrapper::TransactionWrapper;
use anyhow::Result;
use mockall::automock;

/// アートワーク関係のDBリポジトリ
#[automock]
pub trait DbArtworkRepository {
    /// 曲に紐づくアートワークの情報を取得する
    /// # Arguments
    /// - song_id: アートワーク情報を取得する曲のID
    /// # Returns
    /// 指定された曲に紐づく全アートワークの情報
    fn get_song_artworks<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
    ) -> Result<Vec<SongArtwork>>;

    /// 曲にアートワークの紐付きを登録
    ///
    /// orderは無視し、関数内で上書きする。
    ///
    /// # Arguments
    /// - song_id: 紐付けを登録する曲のID
    /// - song_artworks: 曲に紐づく全てのアートワークの情報
    fn register_song_artworks<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
        song_artworks: &[SongArtwork],
    ) -> Result<()>;

    /// 曲へのアートワーク紐付き情報を削除
    ///
    /// どの曲にも紐付かないアートワークは、DBから削除する
    ///
    /// # Arguments
    /// - song_id 紐付けを削除する曲のID
    fn unregister_song_artworks<'c>(&self, tx: &TransactionWrapper<'c>, song_id: i32)
    -> Result<()>;
}
