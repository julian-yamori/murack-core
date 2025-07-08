use crate::db_wrapper::TransactionWrapper;
use anyhow::Result;
use mockall::automock;

/// 曲とタグの紐づけ関係のリポジトリ
#[automock]
pub trait DbSongTagRepository {
    /// 曲から全てのタグを削除
    fn delete_all_tags_from_song<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
    ) -> Result<()>;
}
