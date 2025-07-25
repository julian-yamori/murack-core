use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{
    db::DbTransaction,
    playlist::{DbPlaylistTrackRepository, PlaylistType, SortType},
};

use crate::playlist::playlist_row::PlaylistRow;

use super::playlist_sqls;

/// DbPlaylistTrackRepositoryの本実装
#[derive(new)]
pub struct DbPlaylistTrackRepositoryImpl {}

#[async_trait]
impl DbPlaylistTrackRepository for DbPlaylistTrackRepositoryImpl {
    //曲を全プレイリストから削除
    async fn delete_track_from_all_playlists<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        track_id: i32,
    ) -> Result<()> {
        let playlist_rows = sqlx::query_as!(PlaylistRow, r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name, parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists"#)
            .fetch_all(&mut **tx.get())
            .await?;

        //全てのプレイリストについて繰り返す
        for plist in playlist_rows {
            //プレイリスト内の曲を取得
            let tracks = playlist_sqls::select_track_id_by_playlist_id(tx, plist.id).await?;

            //プレイリストから一旦全削除
            playlist_sqls::delete_by_playlist_id(tx, plist.id).await?;

            //削除対象の曲を除き、全て追加
            let add_tracks = tracks.iter().filter(|i| **i != track_id).enumerate();
            for (order, it) in add_tracks {
                playlist_sqls::insert_playlist_track(tx, plist.id, *it, order as i32).await?;
            }
        }

        Ok(())
    }
}
