pub mod with_playlist;
use thiserror::Error;
pub use with_playlist::SortTypeWithPlaylist;

use serde::{Deserialize, Serialize};

/// 曲のソートの種類
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortType {
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

impl SortType {
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

#[derive(Debug, Error)]
#[error("Unknown sort type: {}", .0)]
pub struct UnknownSortType(String);

pub fn asc_or_desc_query(desc: bool) -> &'static str {
    match desc {
        false => "ASC",
        true => "DESC",
    }
}
