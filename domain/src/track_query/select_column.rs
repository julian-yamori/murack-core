use sqlx::{Row, postgres::PgRow};

use crate::path::LibraryTrackPath;

/// `tracks` テーブルからの検索時に取得するカラム
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SelectColumn {
    Id,
    Path,
    Title,
    ArtworkId,
}

impl SelectColumn {
    /// PgRow から曲の ID を取得
    pub fn row_id(row: &PgRow) -> sqlx::Result<i32> {
        row.try_get("id")
    }

    /// PgRow から曲のパスを取得
    pub fn row_path(row: &PgRow) -> sqlx::Result<LibraryTrackPath> {
        row.try_get("path")
    }

    /// PgRow から曲名を取得
    pub fn row_title(row: &PgRow) -> sqlx::Result<String> {
        row.try_get("title")
    }

    /// PgRow からアートワーク ID を取得
    pub fn row_artwork_id(row: &PgRow) -> sqlx::Result<Option<i32>> {
        row.try_get("artwork_id")
    }

    pub(super) fn sql_column_name(&self) -> &'static str {
        match self {
            Self::Id => "tracks.id",
            Self::Path => "tracks.path",
            Self::Title => "tracks.title",
            Self::ArtworkId => "track_artworks.artwork_id",
        }
    }
}
