use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;
use sqlx::PgTransaction;

use super::Playlist;

/// プレイリスト関係のDBリポジトリ
#[async_trait]
pub trait DbPlaylistRepository {
    /// IDを指定してプレイリストを検索
    /// # Arguments
    /// id: playlist.rowid
    async fn get_playlist<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        id: i32,
    ) -> Result<Option<Playlist>>;

    /// プレイリストのツリー構造を取得
    /// # Returns
    /// 最上位プレイリストのリスト
    async fn get_playlist_tree<'c>(&self, tx: &mut PgTransaction<'c>) -> Result<Vec<Playlist>>;

    /// 全フィルタプレイリスト・フォルダプレイリストの、リストアップ済みフラグを解除する。
    async fn reset_listuped_flag<'c>(&self, tx: &mut PgTransaction<'c>) -> Result<()>;

    /// 全プレイリストの、DAP に保存してからの変更フラグを設定
    /// # Arguments
    /// - is_changed: 変更されたか
    async fn set_dap_change_flag_all<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        is_changed: bool,
    ) -> Result<()>;
}

#[derive(Default)]
pub struct MockDbPlaylistRepository {
    pub inner: MockDbPlaylistRepositoryInner,
}
#[async_trait]
impl DbPlaylistRepository for MockDbPlaylistRepository {
    async fn get_playlist<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        id: i32,
    ) -> Result<Option<Playlist>> {
        self.inner.get_playlist(id)
    }

    async fn get_playlist_tree<'c>(&self, _db: &mut PgTransaction<'c>) -> Result<Vec<Playlist>> {
        self.inner.get_playlist_tree()
    }

    async fn reset_listuped_flag<'c>(&self, _db: &mut PgTransaction<'c>) -> Result<()> {
        self.inner.reset_listuped_flag()
    }

    async fn set_dap_change_flag_all<'c>(
        &self,
        _db: &mut PgTransaction<'c>,
        is_changed: bool,
    ) -> Result<()> {
        self.inner.set_dap_change_flag_all(is_changed)
    }
}
mock! {
    pub DbPlaylistRepositoryInner {
        pub fn get_playlist(&self, id: i32) -> Result<Option<Playlist>>;

        pub fn get_playlist_tree(&self) -> Result<Vec<Playlist>>;

        pub fn reset_listuped_flag(&self) -> Result<()>;

        pub fn set_dap_change_flag_all(
            &self,
            is_changed: bool,
        ) -> Result<()>;
    }
}
