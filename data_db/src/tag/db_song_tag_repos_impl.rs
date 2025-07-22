use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{db::DbTransaction, tag::DbSongTagRepository};

use super::SongTagsDao;

/// DbSongTagRepositoryの本実装
#[derive(new)]
pub struct DbSongTagRepositoryImpl<STD>
where
    STD: SongTagsDao + Sync + Send,
{
    song_tags_dao: STD,
}

#[async_trait]
impl<STD> DbSongTagRepository for DbSongTagRepositoryImpl<STD>
where
    STD: SongTagsDao + Sync + Send,
{
    /// 曲から全てのタグを削除
    async fn delete_all_tags_from_song<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()> {
        self.song_tags_dao.delete_by_song_id(tx, song_id).await
    }
}
