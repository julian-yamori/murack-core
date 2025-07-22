use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::db::DbTransaction;

/// playlist_songテーブルのDAO
#[async_trait]
pub trait PlaylistSongDao {
    /// プレイリストIDを指定して曲IDを取得
    async fn select_song_id_by_playlist_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        plist_id: i32,
    ) -> Result<Vec<i32>>;

    /// 新規登録
    async fn insert<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        plist_id: i32,
        song_id: i32,
        order: i32,
    ) -> Result<()>;

    /// プレイリストIDを指定して削除
    ///
    /// # Arguments
    /// - plist_id: 削除元のプレイリストのID
    async fn delete_by_playlist_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        plist_id: i32,
    ) -> Result<()>;
}

/// PlaylistSongDaoの本実装
pub struct PlaylistSongDaoImpl {}

#[async_trait]
impl PlaylistSongDao for PlaylistSongDaoImpl {
    /// プレイリストIDを指定して曲IDを取得
    async fn select_song_id_by_playlist_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        plist_id: i32,
    ) -> Result<Vec<i32>> {
        let id = sqlx::query_scalar!(
            "SELECT track_id FROM playlist_tracks WHERE playlist_id = $1",
            plist_id,
        )
        .fetch_all(&mut **tx.get())
        .await?;

        Ok(id)
    }

    /// 新規登録
    async fn insert<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        plist_id: i32,
        song_id: i32,
        order: i32,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO playlist_tracks (playlist_id, order_index, track_id) VALUES($1, $2, $3)",
            plist_id,
            order,
            song_id,
        )
        .execute(&mut **tx.get())
        .await?;

        Ok(())
    }

    /// プレイリストIDを指定して削除
    ///
    /// # Arguments
    /// - plist_id: 削除元のプレイリストのID
    async fn delete_by_playlist_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        plist_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            "DELETE FROM playlist_tracks WHERE playlist_id = $1",
            plist_id,
        )
        .execute(&mut **tx.get())
        .await?;

        Ok(())
    }
}
