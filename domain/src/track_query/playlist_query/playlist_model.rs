use sqlx::PgTransaction;

use crate::{
    SortTypeWithPlaylist,
    filter::RootFilter,
    playlist::{PlaylistType, playlist_error::PlaylistError},
};

/// playlist_query モジュールで使用する、プレイリストデータのモデル
#[derive(Debug, PartialEq)]
pub struct QueryPlaylistModel {
    /// プレイリストID
    pub id: i32,

    /// プレイリストの種類
    pub playlist_type: PlaylistType,

    /// PlaylistType::Filter で使うフィルタ
    pub filter: Option<RootFilter>,

    /// ソート対象
    pub sort_type: SortTypeWithPlaylist,

    /// ソートが降順か
    pub sort_desc: bool,

    /// リスト内容がPlaylistTrackテーブルにリストアップ済みか
    ///
    /// 更新されうる処理が行われるごとに、
    /// FilterとFolderのフラグが解除される。
    ///
    /// Normalでは常にtrue。
    pub listuped_flag: bool,
}

impl QueryPlaylistModel {
    pub async fn from_db<'c>(
        tx: &mut PgTransaction<'c>,
        playlist_id: i32,
    ) -> Result<QueryPlaylistModel, PlaylistError> {
        let row = sqlx::query_as!(
            PlaylistRow,
            r#"
            SELECT
              id,
              playlist_type AS "playlist_type: PlaylistType",
              filter_json,
              sort_type AS "sort_type: SortTypeWithPlaylist",
              sort_desc,
              listuped_flag
            FROM playlists
            WHERE id = $1
            "#,
            playlist_id
        )
        .fetch_one(&mut **tx)
        .await?;

        row.try_into()
    }

    pub async fn from_db_by_parent<'c>(
        tx: &mut PgTransaction<'c>,
        parent_id: i32,
    ) -> Result<Vec<QueryPlaylistModel>, PlaylistError> {
        let playlists = sqlx::query_as!(
            PlaylistRow,
            r#"
            SELECT
              id,
              playlist_type AS "playlist_type: PlaylistType",
              filter_json,
              sort_type AS "sort_type: SortTypeWithPlaylist",
              sort_desc,
              listuped_flag
            FROM playlists
            WHERE parent_id = $1
            ORDER BY in_folder_order
            "#,
            parent_id
        )
        .map(QueryPlaylistModel::try_from)
        .fetch_all(&mut **tx)
        .await?
        .into_iter()
        .collect::<Result<Vec<_>, PlaylistError>>()?;

        Ok(playlists)
    }
}

/// QueryPlaylistModel についての、playlist テーブルのレコード
struct PlaylistRow {
    pub id: i32,
    pub playlist_type: PlaylistType,
    pub filter_json: Option<serde_json::Value>,
    pub sort_type: SortTypeWithPlaylist,
    pub sort_desc: bool,
    pub listuped_flag: bool,
}

impl TryFrom<PlaylistRow> for QueryPlaylistModel {
    type Error = PlaylistError;

    fn try_from(row: PlaylistRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            playlist_type: row.playlist_type,
            filter: match row.filter_json {
                Some(json) => Some(
                    serde_json::from_value(json)
                        .map_err(PlaylistError::FailedToDeserializeFilter)?,
                ),
                None => None,
            },
            sort_type: row.sort_type,
            sort_desc: row.sort_desc,
            listuped_flag: row.listuped_flag,
        })
    }
}
