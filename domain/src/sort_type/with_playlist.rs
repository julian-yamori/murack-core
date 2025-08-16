use std::{error::Error, str::FromStr};

use serde::{Deserialize, Serialize};
use sqlx::{
    Postgres,
    encode::IsNull,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef},
};

use crate::sort_type::{SortType, UnknownSortType};

/// 曲のソートの種類 (プレイリスト順付き)
///
/// 通常プレイリストでのみ使用可能
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SortTypeWithPlaylist {
    /// プレイリストでの並び順
    Playlist,

    General(SortType),
}

impl SortTypeWithPlaylist {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Playlist => "playlist",
            Self::General(t) => match *t {
                SortType::TrackName => "track_name",
                SortType::Artist => "artist",
                SortType::Album => "album",
                SortType::Genre => "genre",
                SortType::Composer => "composer",
                SortType::Duration => "duration",
                SortType::TrackIndex => "track_index",
                SortType::DiscIndex => "disc_index",
                SortType::ReleaseDate => "release_date",
                SortType::Rating => "rating",
                SortType::EntryDate => "entry_date",
                SortType::Path => "path",
            },
        }
    }

    /// カラムのソート順のクエリを取得
    ///
    /// - is_desc: ソートが降順か
    /// - playlist_track_index_column: `playlist_tracks.order_index` カラムの名前
    ///
    /// `title_order ASC, tracks.id DESC` の形式の文字列を返す
    pub fn order_query(&self, desc: bool, playlist_track_index_column: &str) -> String {
        match self {
            Self::General(t) => t.order_query(desc),

            Self::Playlist => {
                let dir = super::asc_or_desc_query(desc);
                format!("{playlist_track_index_column} {dir}")
            }
        }
    }
}

impl FromStr for SortTypeWithPlaylist {
    type Err = UnknownSortType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "playlist" => Ok(Self::Playlist),

            "track_name" => Ok(SortType::TrackName.into()),
            "artist" => Ok(SortType::Artist.into()),
            "album" => Ok(SortType::Album.into()),
            "genre" => Ok(SortType::Genre.into()),
            "composer" => Ok(SortType::Composer.into()),
            "duration" => Ok(SortType::Duration.into()),
            "track_index" => Ok(SortType::TrackIndex.into()),
            "disc_index" => Ok(SortType::DiscIndex.into()),
            "release_date" => Ok(SortType::ReleaseDate.into()),
            "rating" => Ok(SortType::Rating.into()),
            "entry_date" => Ok(SortType::EntryDate.into()),
            "path" => Ok(SortType::Path.into()),

            _ => Err(UnknownSortType(s.to_string())),
        }
    }
}

impl From<SortType> for SortTypeWithPlaylist {
    fn from(value: SortType) -> Self {
        Self::General(value)
    }
}

impl Serialize for SortTypeWithPlaylist {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Playlist => "playlist".serialize(serializer),
            Self::General(sort_type) => sort_type.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for SortTypeWithPlaylist {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let t = Self::from_str(&s).map_err(serde::de::Error::custom)?;
        Ok(t)
    }
}

impl sqlx::Type<Postgres> for SortTypeWithPlaylist {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("sort_type_with_playlist")
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for SortTypeWithPlaylist {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<Postgres>>::decode(value)?;

        Ok(Self::from_str(&s)?)
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for SortTypeWithPlaylist {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        <&str as sqlx::Encode<Postgres>>::encode_by_ref(&self.as_str(), buf)
    }
}

#[cfg(feature = "openapi")]
impl utoipa::ToSchema for SortTypeWithPlaylist {}

#[cfg(feature = "openapi")]
impl utoipa::PartialSchema for SortTypeWithPlaylist {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .description(Some("曲のソートの種類 (プレイリスト順付き)"))
            .enum_values(Some([
                "playlist",
                "track_name",
                "artist",
                "album",
                "genre",
                "composer",
                "duration",
                "track_index",
                "disc_index",
                "release_date",
                "rating",
                "entry_date",
                "path",
            ]))
            .into()
    }
}
