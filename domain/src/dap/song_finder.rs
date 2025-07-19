use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use crate::{db::DbTransaction, path::LibSongPath, playlist::Playlist};

/// 曲データの検索機能
/// #todo
/// 現状ではWalkBaseでの用途が限定的だし、
/// やたらとごついし、
/// 整理したいやつ。
///
/// とりあえずdapモジュールに定義
#[async_trait]
pub trait SongFinder {
    /// プレイリストに含まれる曲のパスリストを取得
    /// # Arguments
    /// - plist 取得対象のプレイリスト情報
    async fn get_song_path_list<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        plist: &Playlist,
    ) -> Result<Vec<LibSongPath>>;
}

#[derive(Default)]
pub struct MockSongFinder {
    pub inner: MockSongFinderInner,
}
#[async_trait]
impl SongFinder for MockSongFinder {
    async fn get_song_path_list<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        plist: &Playlist,
    ) -> Result<Vec<LibSongPath>> {
        self.inner.get_song_path_list(plist)
    }
}
mock! {
    pub SongFinderInner {
        pub fn get_song_path_list(
            &self,
            plist: &Playlist,
        ) -> Result<Vec<LibSongPath>>;
    }
}
