use super::SongTagsDao;
use anyhow::Result;
use domain::{db_wrapper::TransactionWrapper, tag::DbSongTagRepository};
use std::rc::Rc;

/// DbSongTagRepositoryの本実装
#[derive(new)]
pub struct DbSongTagRepositoryImpl {
    song_tags_dao: Rc<dyn SongTagsDao>,
}

impl DbSongTagRepository for DbSongTagRepositoryImpl {
    /// 曲から全てのタグを削除
    fn delete_all_tags_from_song(&self, tx: &TransactionWrapper, song_id: i32) -> Result<()> {
        self.song_tags_dao.delete_by_song_id(tx, song_id)
    }
}
