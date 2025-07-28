use murack_core_domain::NonEmptyString;

/// プレイリスト関連のエラー
#[derive(thiserror::Error, Debug)]
pub enum PlaylistError {
    #[error("親が見つからないプレイリストが検出されました: {}", diaplay_playlist_no_parents_detected(.0))]
    PlaylistNoParentsDetected(Vec<PlaylistNoParentsDetectedItem>),

    #[error("フィルタプレイリストにフィルタがありません: playlist_id={plist_id}")]
    FilterPlaylistHasNoFilter { plist_id: i32 },
}

#[derive(Debug, PartialEq)]
pub struct PlaylistNoParentsDetectedItem {
    pub playlist_id: i32,
    pub name: NonEmptyString,
    pub parent_id: Option<i32>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diaplay_playlist_no_parents_detected() -> anyhow::Result<()> {
        assert_eq!(
            &diaplay_playlist_no_parents_detected(&[
                PlaylistNoParentsDetectedItem {
                    playlist_id: 4,
                    name: "test1".to_string().try_into()?,
                    parent_id: Some(2),
                },
                PlaylistNoParentsDetectedItem {
                    playlist_id: 15,
                    name: "hoge".to_string().try_into()?,
                    parent_id: None,
                }
            ]),
            "{id: 4, name: test1, parent_id: 2}, {id: 15, name: hoge, parent_id: None}"
        );

        Ok(())
    }
}
