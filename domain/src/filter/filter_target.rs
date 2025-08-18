use serde::{Deserialize, Serialize};

use crate::filter::{
    ArtworkFilterRange, BoolFilterRange, DateFilterRange, GroupOperand, IntFilterRange,
    StringFilterRange, TagsFilterRange, range_group,
};

/// フィルタの対象の項目と、その項目に対応した条件情報
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "target")]
pub enum FilterTarget {
    /// 集合フィルタ
    #[serde(rename = "group")]
    FilterGroup {
        op: GroupOperand,
        children: Vec<FilterTarget>,
    },

    /// タグ (track_tags.tag_id)
    #[serde(rename = "tags")]
    Tags { range: TagsFilterRange },

    /// レート (rating)
    #[serde(rename = "rating")]
    Rating { range: IntFilterRange },

    /// ジャンル (genre)
    #[serde(rename = "genre")]
    Genre { range: StringFilterRange },

    /// アーティスト (artist)
    #[serde(rename = "artist")]
    Artist { range: StringFilterRange },

    /// アルバムアーティスト (album_artist)
    #[serde(rename = "albumartist")]
    Albumartist { range: StringFilterRange },

    /// アルバム (album)
    #[serde(rename = "album")]
    Album { range: StringFilterRange },

    /// 作曲者 (composer)
    #[serde(rename = "composer")]
    Composer { range: StringFilterRange },

    /// 曲名 (title)
    #[serde(rename = "title")]
    Title { range: StringFilterRange },

    /// アートワーク (track_artworkテーブル)
    #[serde(rename = "artwork")]
    Artwork { range: ArtworkFilterRange },

    /// 再生時間 (duration)
    ///
    /// `tracks.duration` と同じく、IntFilterRange の値にはミリ秒の i32 を格納する。
    ///
    /// 画面からの入力は「分:秒」形式
    #[serde(rename = "duration")]
    Duration { range: IntFilterRange },

    /// リリース日 (release_date)
    #[serde(rename = "release_date")]
    ReleaseDate { range: DateFilterRange },

    /// トラック番号 (track_number)
    #[serde(rename = "track_number")]
    TrackNumber { range: IntFilterRange },

    /// トラック最大数 (track_max)
    #[serde(rename = "track_max")]
    TrackMax { range: IntFilterRange },

    /// ディスク番号 (disc_number)
    #[serde(rename = "disc_number")]
    DiscNumber { range: IntFilterRange },

    /// ディスク最大数 (disc_max)
    #[serde(rename = "disc_max")]
    DiscMax { range: IntFilterRange },

    /// メモ (memo)
    #[serde(rename = "memo")]
    Memo { range: StringFilterRange },

    /// 管理メモ (memo_manage)
    #[serde(rename = "memo_manage")]
    MemoManage { range: StringFilterRange },

    /// 登録日 (entry_date)
    #[serde(rename = "entry_date")]
    EntryDate { range: DateFilterRange },

    /// 原曲 (original_track)
    #[serde(rename = "original_track")]
    OriginalTrack { range: StringFilterRange },

    /// サジェスト対象 (suggest_target)
    #[serde(rename = "suggest_target")]
    SuggestTarget { range: BoolFilterRange },
}

impl FilterTarget {
    /// SQL の WHERE で使用する条件式に変換
    ///
    /// フィルタ条件が無い場合は None (空の Group しか無い場合)
    pub fn where_expression(&self) -> Option<String> {
        let some = match self {
            FilterTarget::FilterGroup { op, children } => {
                // Group の場合のみ Option を返すので early return
                return range_group::group_where_expression(op, children);
            }

            FilterTarget::Tags { range } => range.where_expression(),
            FilterTarget::Rating { range } => range.where_expression("rating"),
            FilterTarget::Genre { range } => range.where_expression("genre"),
            FilterTarget::Artist { range } => range.where_expression("artist"),
            FilterTarget::Albumartist { range } => range.where_expression("album_artist"),
            FilterTarget::Album { range } => range.where_expression("album"),
            FilterTarget::Composer { range } => range.where_expression("composer"),
            FilterTarget::Title { range } => range.where_expression("title"),
            FilterTarget::Artwork { range } => range.where_expression(),
            FilterTarget::Duration { range } => range.where_expression("duration"),
            FilterTarget::ReleaseDate { range } => range.where_expression("release_date"),
            FilterTarget::TrackNumber { range } => range.where_expression("track_number"),
            FilterTarget::TrackMax { range } => range.where_expression("track_max"),
            FilterTarget::DiscNumber { range } => range.where_expression("disc_number"),
            FilterTarget::DiscMax { range } => range.where_expression("disc_max"),
            FilterTarget::Memo { range } => range.where_expression("memo"),
            FilterTarget::MemoManage { range } => range.where_expression("memo_manage"),

            // TODO DateTime との比較は危なそう #17
            FilterTarget::EntryDate { range } => range.where_expression("created_at"),

            FilterTarget::OriginalTrack { range } => range.where_expression("original_track"),
            FilterTarget::SuggestTarget { range } => range.where_expression("suggest_target"),
        };

        Some(some)
    }
}
