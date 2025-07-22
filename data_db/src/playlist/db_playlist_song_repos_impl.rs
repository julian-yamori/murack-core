use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{db::DbTransaction, playlist::DbPlaylistSongRepository};

use super::{PlaylistDao, PlaylistSongDao};

/// DbPlaylistSongRepositoryの本実装
#[derive(new)]
pub struct DbPlaylistSongRepositoryImpl<PD, PSD>
where
    PD: PlaylistDao + Sync + Send,
    PSD: PlaylistSongDao + Sync + Send,
{
    playlist_dao: PD,
    playlist_song_dao: PSD,
}

#[async_trait]
impl<PD, PSD> DbPlaylistSongRepository for DbPlaylistSongRepositoryImpl<PD, PSD>
where
    PD: PlaylistDao + Sync + Send,
    PSD: PlaylistSongDao + Sync + Send,
{
    //曲を全プレイリストから削除
    async fn delete_song_from_all_playlists<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()> {
        //全てのプレイリストについて繰り返す
        for plist in self.playlist_dao.select_all(tx).await? {
            let playlist_song_dao = &self.playlist_song_dao;

            //プレイリスト内の曲を取得
            let songs = playlist_song_dao
                .select_song_id_by_playlist_id(tx, plist.id)
                .await?;

            //プレイリストから一旦全削除
            playlist_song_dao
                .delete_by_playlist_id(tx, plist.id)
                .await?;

            //削除対象の曲を除き、全て追加
            let add_songs = songs.iter().filter(|i| **i != song_id).enumerate();
            for (order, it) in add_songs {
                playlist_song_dao
                    .insert(tx, plist.id, *it, order as i32)
                    .await?;
            }
        }

        Ok(())
    }
}
