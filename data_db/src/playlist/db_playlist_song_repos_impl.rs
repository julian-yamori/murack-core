use super::{PlaylistDao, PlaylistSongDao};
use anyhow::Result;
use domain::{db_wrapper::TransactionWrapper, playlist::DbPlaylistSongRepository};
use std::rc::Rc;

/// DbPlaylistSongRepositoryの本実装
#[derive(new)]
pub struct DbPlaylistSongRepositoryImpl {
    playlist_dao: Rc<dyn PlaylistDao>,
    playlist_song_dao: Rc<dyn PlaylistSongDao>,
}

impl DbPlaylistSongRepository for DbPlaylistSongRepositoryImpl {
    //曲を全プレイリストから削除
    fn delete_song_from_all_playlists(&self, tx: &TransactionWrapper, song_id: i32) -> Result<()> {
        //全てのプレイリストについて繰り返す
        for plist in self.playlist_dao.select_all(tx)? {
            let playlist_song_dao = &self.playlist_song_dao;

            //プレイリスト内の曲を取得
            let songs = playlist_song_dao.select_song_id_by_playlist_id(tx, plist.rowid)?;

            //プレイリストから一旦全削除
            playlist_song_dao.delete_by_playlist_id(tx, plist.rowid)?;

            //削除対象の曲を除き、全て追加
            let add_songs = songs.iter().filter(|i| **i != song_id).enumerate();
            for (order, it) in add_songs {
                playlist_song_dao.insert(tx, plist.rowid, *it, order as i32)?;
            }
        }

        Ok(())
    }
}
