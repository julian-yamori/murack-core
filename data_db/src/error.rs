use domain::filter::{FilterTarget, FilterValueRange};

/// WalkBase2 data層のエラー
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // enum変換エラー
    #[error("不正なPlaylistTypeです: {type_num}")]
    InvalidPlaylistType { type_num: i64 },
    #[error("不正なSortTypeです: {type_num}")]
    InvalidSortType { type_num: i64 },
    #[error("不正なFilterTargetです: {type_num}")]
    InvalidFilterTarget { type_num: i64 },
    #[error("不正なFilterValueRangeです: {type_num}")]
    InvalidFilterValueRange { type_num: i64 },

    #[error("親が見つからないプレイリストが検出されました: {}", diaplay_playlist_no_parents_detected(.0))]
    PlaylistNoParentsDetected(Vec<PlaylistNoParentsDetectedItem>),

    #[error("フィルタプレイリストのフィルタIDがNoneです: playlist_id={plist_id}")]
    FilterPlaylistFilterIdNone { plist_id: i32 },
    #[error("rootフィルタが見つかりませんでした: root_id={root_id}")]
    RootFilterNotFound { root_id: i32 },
    #[error("親がみつからないフィルタが検出されました: {}", diaplay_filter_no_parents_detected(.0))]
    FilterNoParentsDetected(Vec<FilterNoParentsDetectedItem>),
    #[error("FilterRangeの値が不正です: filter_id={filter_id}, target={target}, range={range}")]
    InvalidFilterRangeForTarget {
        filter_id: i32,
        target: FilterTarget,
        range: FilterValueRange,
    },
}

#[derive(Debug, PartialEq)]
pub struct PlaylistNoParentsDetectedItem {
    pub playlist_id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
}

#[derive(Debug, PartialEq)]
pub struct FilterNoParentsDetectedItem {
    pub filter_id: i32,
    pub parent_id: Option<i32>,
    pub root_id: i32,
}

fn diaplay_playlist_no_parents_detected(items: &[PlaylistNoParentsDetectedItem]) -> String {
    let v: Vec<String> = items
        .iter()
        .map(|i| {
            format!(
                "{{id: {}, name: {}, parent_id: {}}}",
                i.playlist_id,
                i.name,
                i.parent_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "None".to_owned())
            )
        })
        .collect();
    v.join(", ")
}

fn diaplay_filter_no_parents_detected(items: &[FilterNoParentsDetectedItem]) -> String {
    let v: Vec<String> = items
        .iter()
        .map(|i| {
            format!(
                "{{id: {}, parent_id: {}, root_id: {}}}",
                i.filter_id,
                i.parent_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "None".to_owned()),
                i.root_id,
            )
        })
        .collect();
    v.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diaplay_playlist_no_parents_detected() {
        assert_eq!(
            &diaplay_playlist_no_parents_detected(&[
                PlaylistNoParentsDetectedItem {
                    playlist_id: 4,
                    name: "test1".to_owned(),
                    parent_id: Some(2),
                },
                PlaylistNoParentsDetectedItem {
                    playlist_id: 15,
                    name: "hoge".to_owned(),
                    parent_id: None,
                }
            ]),
            "{id: 4, name: test1, parent_id: 2}, {id: 15, name: hoge, parent_id: None}"
        )
    }
}
