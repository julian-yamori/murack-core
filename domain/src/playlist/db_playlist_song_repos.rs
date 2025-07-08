use crate::db_wrapper::TransactionWrapper;
use anyhow::Result;
use mockall::automock;

/// 曲とプレイリストの紐づけ関係のDBリポジトリ
#[automock]
pub trait DbPlaylistSongRepository {
    //曲を全プレイリストから削除
    fn delete_song_from_all_playlists<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
    ) -> Result<()>;
}
