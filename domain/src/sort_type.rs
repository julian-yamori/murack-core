use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use thiserror::Error;

/// 曲のソートの種類
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "sort_type_with_playlist", rename_all = "lowercase")]
pub enum SortType {
    /// 曲名
    TrackName,
    /// アーティスト
    Artist,
    /// アルバム
    Album,
    /// ジャンル
    Genre,
    /// プレイリストでの並び順
    Playlist,
    /// 作曲者
    Composer,
    /// 再生時間
    Duration,
    /// トラック番号
    TrackIndex,
    /// ディスク番号
    DiscIndex,
    /// リリース日
    ReleaseDate,
    /// レート
    Rating,
    /// 登録日
    EntryDate,
    /// パス
    Path,
}

impl SortType {
    /// カラムのソート順のクエリを取得
    ///
    /// - is_desc: ソートが降順か
    /// - playlist_track_index_column: `playlist_tracks.order_index` カラムの名前
    ///
    /// `title_order ASC, tracks.id DESC` の形式の文字列を返す
    pub fn order_query(&self, desc: bool, playlist_track_index_column: &str) -> String {
        match SortTypeWithoutPlaylist::try_from(*self) {
            // プレイリスト以外は SortTypeWithoutPlaylist::order_query() を返す
            Ok(t) => t.order_query(desc),

            // プレイリストの場合
            Err(SortTypeIsPlaylistError) => {
                let dir = asc_or_desc_query(desc);
                format!("{playlist_track_index_column} {dir}")
            }
        }
    }
}

// TODO この方がいいと思う
// pub enum SortTypeWithPlaylist {
//     General(SortType),
//     Playlist,
// }

/// 曲のソートの種類 (プレイリスト順を除く)
///
/// プレイリスト以外では SortType ではなくこちらを使う
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum SortTypeWithoutPlaylist {
    /// 曲名
    TrackName,
    /// アーティスト
    Artist,
    /// アルバム
    Album,
    /// ジャンル
    Genre,
    /// 作曲者
    Composer,
    /// 再生時間
    Duration,
    /// トラック番号
    TrackIndex,
    /// ディスク番号
    DiscIndex,
    /// リリース日
    ReleaseDate,
    /// レート
    Rating,
    /// 登録日
    EntryDate,
    /// パス
    Path,
}

impl SortTypeWithoutPlaylist {
    /// カラムのソート順のクエリを取得
    ///
    /// - is_desc: ソートが降順か
    ///
    /// `title_order ASC, tracks.id DESC` の形式の文字列を返す
    pub fn order_query(&self, desc: bool) -> String {
        let dir = asc_or_desc_query(desc);

        match self {
            Self::TrackName => format!("title_order {dir}, tracks.id {dir}"),
            Self::Artist => format!(
                "artist_order {dir}, album_order {dir}, disc_number {dir}, track_number {dir}, title_order {dir}, tracks.id {dir}"
            ),
            Self::Album => format!(
                "album_order {dir}, artist_order {dir}, disc_number {dir}, track_number {dir}, title_order {dir}, tracks.id {dir}"
            ),
            Self::Genre => format!(
                "genre {dir}, artist_order {dir}, album_order {dir}, disc_number {dir}, track_number {dir}, title_order {dir}, trakcs.id {dir}"
            ),
            Self::Composer => format!(
                "composer_order {dir}, artist_order {dir}, album_order {dir}, disc_number {dir}, track_number {dir}, title_order {dir}, tracks.id {dir}"
            ),
            Self::Duration => format!("duration {dir}, title_order {dir}, tracks.id {dir}"),
            Self::TrackIndex => format!(
                "track_number {dir}, artist_order {dir}, album_order {dir}, disc_number {dir}, title_order {dir}, tracks.id {dir}"
            ),
            Self::DiscIndex => format!(
                "disc_number {dir}, artist_order {dir}, album_order {dir}, track_number {dir}, title_order {dir}, tracks.id {dir}"
            ),
            Self::ReleaseDate => format!(
                "release_date {dir}, artist_order {dir}, album_order {dir}, disc_number {dir}, track_number {dir}, title_order {dir}, tracks.id {dir}"
            ),
            Self::Rating => format!(
                "rating {dir}, artist_order {dir}, album_order {dir}, disc_number {dir}, track_number {dir}, title_order {dir}, tracks.id {dir}"
            ),
            Self::EntryDate => format!("created_at {dir}, path {dir}"),
            Self::Path => format!("path {dir}"),
        }
    }
}

impl From<SortTypeWithoutPlaylist> for SortType {
    fn from(value: SortTypeWithoutPlaylist) -> Self {
        match value {
            SortTypeWithoutPlaylist::TrackName => Self::TrackName,
            SortTypeWithoutPlaylist::Artist => Self::Artist,
            SortTypeWithoutPlaylist::Album => Self::Album,
            SortTypeWithoutPlaylist::Genre => Self::Genre,
            SortTypeWithoutPlaylist::Composer => Self::Composer,
            SortTypeWithoutPlaylist::Duration => Self::Duration,
            SortTypeWithoutPlaylist::TrackIndex => Self::TrackIndex,
            SortTypeWithoutPlaylist::DiscIndex => Self::DiscIndex,
            SortTypeWithoutPlaylist::ReleaseDate => Self::ReleaseDate,
            SortTypeWithoutPlaylist::Rating => Self::Rating,
            SortTypeWithoutPlaylist::EntryDate => Self::EntryDate,
            SortTypeWithoutPlaylist::Path => Self::Path,
        }
    }
}

impl TryFrom<SortType> for SortTypeWithoutPlaylist {
    type Error = SortTypeIsPlaylistError;

    fn try_from(value: SortType) -> Result<Self, Self::Error> {
        match value {
            SortType::TrackName => Ok(Self::TrackName),
            SortType::Artist => Ok(Self::Artist),
            SortType::Album => Ok(Self::Album),
            SortType::Genre => Ok(Self::Genre),
            SortType::Composer => Ok(Self::Composer),
            SortType::Duration => Ok(Self::Duration),
            SortType::TrackIndex => Ok(Self::TrackIndex),
            SortType::DiscIndex => Ok(Self::DiscIndex),
            SortType::ReleaseDate => Ok(Self::ReleaseDate),
            SortType::Rating => Ok(Self::Rating),
            SortType::EntryDate => Ok(Self::EntryDate),
            SortType::Path => Ok(Self::Path),

            SortType::Playlist => Err(SortTypeIsPlaylistError),
        }
    }
}

#[derive(Debug, Error)]
#[error("SortType::Playlist cannot converted into SortTypeWithoutPlaylist")]
pub struct SortTypeIsPlaylistError;

pub fn asc_or_desc_query(desc: bool) -> &'static str {
    match desc {
        false => "ASC",
        true => "DESC",
    }
}
