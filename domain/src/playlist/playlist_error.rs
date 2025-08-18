/// プレイリスト関連のエラー
#[derive(thiserror::Error, Debug)]
pub enum PlaylistError {
    #[error("フィルタの deserialize に失敗しました: {}", .0)]
    FailedToDeserializeFilter(serde_json::Error),

    #[error("フィルタプレイリストにフィルタがありません: playlist_id={plist_id}")]
    FilterPlaylistHasNoFilter { plist_id: i32 },

    #[error("親が見つからないプレイリストが検出されました: {}", diaplay_playlist_no_parents_detected(.0))]
    PlaylistNoParentsDetected(Vec<PlaylistNoParentsDetectedItem>),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, PartialEq)]
pub struct PlaylistNoParentsDetectedItem {
    pub playlist_id: i32,
    pub parent_id: Option<i32>,
}

fn diaplay_playlist_no_parents_detected(items: &[PlaylistNoParentsDetectedItem]) -> String {
    let v: Vec<String> = items
        .iter()
        .map(|i| {
            format!(
                "{{id: {}, parent_id: {}}}",
                i.playlist_id,
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
                    parent_id: Some(2),
                },
                PlaylistNoParentsDetectedItem {
                    playlist_id: 15,
                    parent_id: None,
                }
            ]),
            "{id: 4, parent_id: 2}, {id: 15, parent_id: None}"
        );

        Ok(())
    }
}
