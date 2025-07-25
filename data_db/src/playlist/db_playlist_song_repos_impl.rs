use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{db::DbTransaction, playlist::DbPlaylistSongRepository};

use super::playlist_sqls;

/// DbPlaylistSongRepositoryの本実装
#[derive(new)]
pub struct DbPlaylistSongRepositoryImpl {}

#[async_trait]
impl DbPlaylistSongRepository for DbPlaylistSongRepositoryImpl {
    //曲を全プレイリストから削除
    async fn delete_song_from_all_playlists<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
    ) -> Result<()> {
        //全てのプレイリストについて繰り返す
        for plist in playlist_sqls::select_all_playlists(tx).await? {
            //プレイリスト内の曲を取得
            let songs = playlist_sqls::select_song_id_by_playlist_id(tx, plist.id).await?;

            //プレイリストから一旦全削除
            playlist_sqls::delete_by_playlist_id(tx, plist.id).await?;

            //削除対象の曲を除き、全て追加
            let add_songs = songs.iter().filter(|i| **i != song_id).enumerate();
            for (order, it) in add_songs {
                playlist_sqls::insert_playlist_song(tx, plist.id, *it, order as i32).await?;
            }
        }

        Ok(())
    }
}
