use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use crate::{db::DbTransaction, path::LibTrackPath, playlist::Playlist};

/// 曲データの検索機能
#[async_trait]
pub trait TrackFinder {
    /// プレイリストに含まれる曲のパスリストを取得
    /// # Arguments
    /// - plist 取得対象のプレイリスト情報
    async fn get_track_path_list<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        plist: &Playlist,
    ) -> Result<Vec<LibTrackPath>>;
}

#[derive(Default)]
pub struct MockTrackFinder {
    pub inner: MockTrackFinderInner,
}
#[async_trait]
impl TrackFinder for MockTrackFinder {
    async fn get_track_path_list<'c>(
        &self,
        _db: &mut DbTransaction<'c>,
        plist: &Playlist,
    ) -> Result<Vec<LibTrackPath>> {
        self.inner.get_track_path_list(plist)
    }
}
mock! {
    pub TrackFinderInner {
        pub fn get_track_path_list(
            &self,
            plist: &Playlist,
        ) -> Result<Vec<LibTrackPath>>;
    }
}
