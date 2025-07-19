use anyhow::Result;
use async_trait::async_trait;
use domain::{
    db::DbTransaction,
    playlist::{PlaylistType, SortType},
};
use mockall::mock;

use super::PlaylistRow;

/// playlistテーブルのDAO
#[async_trait]
pub trait PlaylistDao {
    /// IDを指定して検索
    async fn select_by_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        plist_id: i32,
    ) -> Result<Option<PlaylistRow>>;

    /// 全レコードを取得
    async fn select_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<Vec<PlaylistRow>>;

    /// 全レコードを取得(in_folder_order順)
    async fn select_all_order_folder<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
    ) -> Result<Vec<PlaylistRow>>;

    /// プレイリストの子プレイリスト一覧を取得
    /// # Arguments
    /// - parent_id: 親プレイリストID(Noneなら最上位のプレイリストを取得)
    /// # Returns
    /// 指定されたプレイリストの子プレイリスト一覧
    async fn get_child_playlists<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        parent_id: Option<i32>,
    ) -> Result<Vec<PlaylistRow>>;
}

/// PlaylistDaoの本実装
pub struct PlaylistDaoImpl {}

#[async_trait]
impl PlaylistDao for PlaylistDaoImpl {
    /// IDを指定して検索
    async fn select_by_id<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        plist_id: i32,
    ) -> Result<Option<PlaylistRow>> {
        let row = sqlx::query_as!(
            PlaylistRow,
            r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name, parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists WHERE id = $1"#,
            plist_id
        )
        .fetch_optional(&mut **tx.get())
        .await?;

        Ok(row)
    }

    /// 全レコードを取得
    async fn select_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<Vec<PlaylistRow>> {
        let rows = sqlx::query_as!(PlaylistRow, r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name, parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists"#)
            .fetch_all(&mut **tx.get())
            .await?;

        Ok(rows)
    }

    /// 全レコードを取得(in_folder_order順)
    async fn select_all_order_folder<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
    ) -> Result<Vec<PlaylistRow>> {
        let rows = sqlx::query_as!(
            PlaylistRow,
            r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name, parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists ORDER BY in_folder_order"#
        )
        .fetch_all(&mut **tx.get())
        .await?;

        Ok(rows)
    }

    /// プレイリストの子プレイリスト一覧を取得
    /// # Arguments
    /// - parent_id: 親プレイリストID(Noneなら最上位のプレイリストを取得)
    /// # Returns
    /// 指定されたプレイリストの子プレイリスト一覧
    async fn get_child_playlists<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        parent_id: Option<i32>,
    ) -> Result<Vec<PlaylistRow>> {
        let rows = sqlx::query_as!(PlaylistRow, r#"SELECT id, playlist_type AS "playlist_type: PlaylistType", name, parent_id, in_folder_order, filter_json, sort_type AS "sort_type: SortType", sort_desc, save_dap ,listuped_flag ,dap_changed FROM playlists WHERE parent_id IS NOT DISTINCT FROM $1 ORDER BY in_folder_order"#, parent_id).fetch_all(&mut **tx.get()).await?;

        Ok(rows)
    }
}

#[derive(Default)]
pub struct MockPlaylistDao {
    pub inner: MockPlaylistDaoInner,
}
#[async_trait]
impl PlaylistDao for MockPlaylistDao {
    async fn select_by_id<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        plist_id: i32,
    ) -> Result<Option<PlaylistRow>> {
        self.inner.select_by_id(plist_id)
    }

    async fn select_all<'c>(&self, _db: &mut DbTransaction<'c>) -> Result<Vec<PlaylistRow>> {
        self.inner.select_all()
    }

    async fn select_all_order_folder<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
    ) -> Result<Vec<PlaylistRow>> {
        self.inner.select_all_order_folder()
    }

    async fn get_child_playlists<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        parent_id: Option<i32>,
    ) -> Result<Vec<PlaylistRow>> {
        self.inner.get_child_playlists(parent_id)
    }
}
mock! {
    pub PlaylistDaoInner {
        pub  fn select_by_id(
            &self,
            plist_id: i32,
        ) -> Result<Option<PlaylistRow>>;

        pub  fn select_all(&self) -> Result<Vec<PlaylistRow>>;

        pub  fn select_all_order_folder(&self) -> Result<Vec<PlaylistRow>>;

        pub  fn get_child_playlists(
            &self,
            parent_id: Option<i32>,
        ) -> Result<Vec<PlaylistRow>>;
    }
}
