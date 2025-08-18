use serde::{Deserialize, Serialize};

/// アートワークで絞り込み
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum ArtworkFilterRange {
    /// アートワークがある
    #[serde(rename = "has")]
    Has,

    /// アートワークが無い
    #[serde(rename = "none")]
    None,
}

impl ArtworkFilterRange {
    /// SQL の WHERE で使用する条件式に変換
    pub fn where_expression(&self) -> String {
        //存在すればtrueのsql
        let base_sql = "EXISTS(SELECT * FROM track_artworks AS a WHERE a.track_id = tracks.id)";

        match self {
            //アートワーク：ある
            ArtworkFilterRange::Has => base_sql.to_owned(),
            //アートワーク：ない
            ArtworkFilterRange::None => format!("NOT {base_sql}"),
        }
    }
}
