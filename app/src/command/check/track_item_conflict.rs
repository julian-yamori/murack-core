use murack_core_domain::{sync::TrackSync, track::TrackItemKind};

/// 曲の項目一つの競合についての機能
pub struct TrackItemConflict {
    /// 競合があった項目
    pub item_kind: TrackItemKind,
}

impl TrackItemConflict {
    /// 項目名を取得
    pub fn item_name(&self) -> &'static str {
        match self.item_kind {
            TrackItemKind::Title => "曲名",
            TrackItemKind::Artist => "アーティスト",
            TrackItemKind::Album => "アルバムアーティスト",
            TrackItemKind::Genre => "アルバム",
            TrackItemKind::AlbumArtist => "ジャンル",
            TrackItemKind::Composer => "作曲者",
            TrackItemKind::TrackNumber => "トラック番号",
            TrackItemKind::TrackMax => "トラック番号最大",
            TrackItemKind::DiscNumber => "ディスク番号",
            TrackItemKind::DiscMax => "ディスク番号最大",
            TrackItemKind::ReleaseDate => "リリース日",
            TrackItemKind::Memo => "メモ",
            TrackItemKind::Lyrics => "歌詞",
        }
    }

    /// TrackSyncの該当する値を表示
    /// # todo
    /// とりあえずString cloneする
    pub fn display_value(&self, track_sync: &TrackSync) -> Option<String> {
        match self.item_kind {
            TrackItemKind::Title => track_sync.title.clone(),
            TrackItemKind::Artist => track_sync.artist.clone(),
            TrackItemKind::Album => track_sync.album.clone(),
            TrackItemKind::Genre => track_sync.genre.clone(),
            TrackItemKind::AlbumArtist => track_sync.album_artist.clone(),
            TrackItemKind::Composer => track_sync.composer.clone(),
            TrackItemKind::TrackNumber => track_sync.track_number.map(|n| n.to_string()),
            TrackItemKind::TrackMax => track_sync.track_max.map(|n| n.to_string()),
            TrackItemKind::DiscNumber => track_sync.disc_number.map(|n| n.to_string()),
            TrackItemKind::DiscMax => track_sync.disc_max.map(|n| n.to_string()),
            TrackItemKind::ReleaseDate => track_sync.release_date.map(|n| n.to_string()),
            TrackItemKind::Memo => track_sync.memo.clone(),
            TrackItemKind::Lyrics => track_sync.lyrics.clone(),
        }
    }

    /// TrackSyncから別のTrackSyncに、該当する値をコピー
    pub fn copy_each_sync(&self, src_sync: &TrackSync, dest_sync: &mut TrackSync) {
        match self.item_kind {
            TrackItemKind::Title => dest_sync.title = src_sync.title.clone(),
            TrackItemKind::Artist => dest_sync.artist = src_sync.artist.clone(),
            TrackItemKind::Album => dest_sync.album = src_sync.album.clone(),
            TrackItemKind::Genre => dest_sync.genre = src_sync.genre.clone(),
            TrackItemKind::AlbumArtist => dest_sync.album_artist = src_sync.album_artist.clone(),
            TrackItemKind::Composer => dest_sync.composer = src_sync.composer.clone(),
            TrackItemKind::TrackNumber => dest_sync.track_number = src_sync.track_number,
            TrackItemKind::TrackMax => dest_sync.track_max = src_sync.track_max,
            TrackItemKind::DiscNumber => dest_sync.disc_number = src_sync.disc_number,
            TrackItemKind::DiscMax => dest_sync.disc_max = src_sync.disc_max,
            TrackItemKind::ReleaseDate => dest_sync.release_date = src_sync.release_date,
            TrackItemKind::Memo => dest_sync.memo = src_sync.memo.clone(),
            TrackItemKind::Lyrics => dest_sync.lyrics = src_sync.lyrics.clone(),
        }
    }
}
