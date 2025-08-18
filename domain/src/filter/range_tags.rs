use serde::{Deserialize, Serialize};

/// タグで絞り込み
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum TagsFilterRange {
    /// 指定されたタグを含む
    #[serde(rename = "contain")]
    Contain { value: i32 },
    /// 指定されたタグを含まない
    #[serde(rename = "not_contain")]
    NotContain { value: i32 },
    /// タグを持たない
    #[serde(rename = "none")]
    None,
}

impl TagsFilterRange {
    /// SQL の WHERE で使用する条件式に変換
    pub fn where_expression(&self) -> String {
        //タグで検索するクエリを取得する関数
        fn get_query_where_by_tag(tag_id: i32) -> String {
            format!(
                "EXISTS(SELECT * FROM track_tags AS t WHERE t.track_id = tracks.id AND t.tag_id = {tag_id})"
            )
        }

        match self {
            //タグ：含む
            TagsFilterRange::Contain { value } => get_query_where_by_tag(*value),
            //タグ：含まない
            TagsFilterRange::NotContain { value } => {
                format!("NOT {}", get_query_where_by_tag(*value))
            }
            //タグ：タグを持たない
            TagsFilterRange::None => {
                "NOT EXISTS(SELECT * FROM track_tags AS t WHERE t.track_id = tracks.id)".to_owned()
            }
        }
    }
}
