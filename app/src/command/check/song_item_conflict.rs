use murack_core_domain::{song::SongItemKind, sync::SongSync};

/// 曲の項目一つの競合についての機能
pub struct SongItemConflict {
    /// 競合があった項目
    pub item_kind: SongItemKind,
}

impl SongItemConflict {
    /// 項目名を取得
    pub fn item_name(&self) -> &'static str {
        match self.item_kind {
            SongItemKind::Title => "曲名",
            SongItemKind::Artist => "アーティスト",
            SongItemKind::Album => "アルバムアーティスト",
            SongItemKind::Genre => "アルバム",
            SongItemKind::AlbumArtist => "ジャンル",
            SongItemKind::Composer => "作曲者",
            SongItemKind::TrackNumber => "トラック番号",
            SongItemKind::TrackMax => "トラック番号最大",
            SongItemKind::DiscNumber => "ディスク番号",
            SongItemKind::DiscMax => "ディスク番号最大",
            SongItemKind::ReleaseDate => "リリース日",
            SongItemKind::Memo => "メモ",
            SongItemKind::Lyrics => "歌詞",
        }
    }

    /// SongSyncの該当する値を表示
    /// # todo
    /// とりあえずString cloneする
    pub fn display_value(&self, song_sync: &SongSync) -> Option<String> {
        match self.item_kind {
            SongItemKind::Title => song_sync.title.clone(),
            SongItemKind::Artist => song_sync.artist.clone(),
            SongItemKind::Album => song_sync.album.clone(),
            SongItemKind::Genre => song_sync.genre.clone(),
            SongItemKind::AlbumArtist => song_sync.album_artist.clone(),
            SongItemKind::Composer => song_sync.composer.clone(),
            SongItemKind::TrackNumber => song_sync.track_number.map(|n| n.to_string()),
            SongItemKind::TrackMax => song_sync.track_max.map(|n| n.to_string()),
            SongItemKind::DiscNumber => song_sync.disc_number.map(|n| n.to_string()),
            SongItemKind::DiscMax => song_sync.disc_max.map(|n| n.to_string()),
            SongItemKind::ReleaseDate => song_sync.release_date.map(|n| n.to_string()),
            SongItemKind::Memo => song_sync.memo.clone(),
            SongItemKind::Lyrics => song_sync.lyrics.clone(),
        }
    }

    /// SongSyncから別のSongSyncに、該当する値をコピー
    pub fn copy_each_sync(&self, src_sync: &SongSync, dest_sync: &mut SongSync) {
        match self.item_kind {
            SongItemKind::Title => dest_sync.title = src_sync.title.clone(),
            SongItemKind::Artist => dest_sync.artist = src_sync.artist.clone(),
            SongItemKind::Album => dest_sync.album = src_sync.album.clone(),
            SongItemKind::Genre => dest_sync.genre = src_sync.genre.clone(),
            SongItemKind::AlbumArtist => dest_sync.album_artist = src_sync.album_artist.clone(),
            SongItemKind::Composer => dest_sync.composer = src_sync.composer.clone(),
            SongItemKind::TrackNumber => dest_sync.track_number = src_sync.track_number,
            SongItemKind::TrackMax => dest_sync.track_max = src_sync.track_max,
            SongItemKind::DiscNumber => dest_sync.disc_number = src_sync.disc_number,
            SongItemKind::DiscMax => dest_sync.disc_max = src_sync.disc_max,
            SongItemKind::ReleaseDate => dest_sync.release_date = src_sync.release_date,
            SongItemKind::Memo => dest_sync.memo = src_sync.memo.clone(),
            SongItemKind::Lyrics => dest_sync.lyrics = src_sync.lyrics.clone(),
        }
    }
}
