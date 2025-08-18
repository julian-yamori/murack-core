//! フィルタオブジェクトと JSON との相互変換テスト

use std::fmt::Debug;

use serde::{Serialize, de::DeserializeOwned};

fn assert_serde<M>(model: M, json: serde_json::Value)
where
    M: Debug + Clone + PartialEq + Serialize + DeserializeOwned,
{
    // test serialise
    assert_eq!(json, serde_json::to_value(model.clone()).unwrap());

    // test deserialize
    assert_eq!(model, serde_json::from_value(json).unwrap());
}

#[test]
fn test_complex_and_or_combination() {
    use chrono::NaiveDate;

    use crate::filter::{
        DateFilterRange, FilterTarget, GroupOperand, IntFilterRange, StringFilterRange,
        TagsFilterRange,
    };

    let date = NaiveDate::from_ymd_opt(2021, 9, 25).unwrap();

    assert_serde(
        FilterTarget::FilterGroup {
            op: GroupOperand::And,
            children: vec![
                FilterTarget::Artist {
                    range: StringFilterRange::Contain {
                        value: "taro".to_owned(),
                    },
                },
                FilterTarget::FilterGroup {
                    op: GroupOperand::Or,
                    children: vec![
                        FilterTarget::Tags {
                            range: TagsFilterRange::Contain { value: 45 },
                        },
                        FilterTarget::Rating {
                            range: IntFilterRange::LargeEqual { value: 4 },
                        },
                        FilterTarget::ReleaseDate {
                            range: DateFilterRange::Equal { value: date },
                        },
                    ],
                },
            ],
        },
        serde_json::json!({
            "target": "group",
            "op": "and",
            "children": [
                {
                    "target": "artist",
                    "range": {
                        "op": "contain",
                        "value": "taro",
                    }
                },
                {
                    "target": "group",
                    "op": "or",
                    "children": [
                        {
                            "target": "tags",
                            "range": {
                                "op": "contain",
                                "value": 45,
                            }
                        },
                        {
                            "target": "rating",
                            "range": {
                                "op": "large_equal",
                                "value": 4,
                            }
                        },
                        {
                            "target": "release_date",
                            "range": {
                                "op": "equal",
                                "value": date,
                            }
                        },
                    ]
                }
            ],
        }),
    );
}

mod test_group_operand {
    use crate::filter::{FilterTarget, GroupOperand, StringFilterRange};

    use super::*;

    #[test]
    fn and() {
        assert_serde(
            FilterTarget::FilterGroup {
                op: GroupOperand::And,
                children: vec![
                    FilterTarget::Artist {
                        range: StringFilterRange::Equal {
                            value: "Artist1".to_string(),
                        },
                    },
                    FilterTarget::Title {
                        range: StringFilterRange::Equal {
                            value: "Title1".to_string(),
                        },
                    },
                ],
            },
            serde_json::json!({
                "target": "group",
                "op": "and",
                "children": [
                    {
                        "target": "artist",
                        "range": {
                            "op": "equal",
                            "value": "Artist1",
                        }
                    },
                    {
                        "target": "title",
                        "range": {
                            "op": "equal",
                            "value": "Title1",
                        }
                    }
                ]
            }),
        );
    }

    #[test]
    fn or() {
        assert_serde(
            FilterTarget::FilterGroup {
                op: GroupOperand::Or,
                children: vec![
                    FilterTarget::Artist {
                        range: StringFilterRange::Equal {
                            value: "Artist1".to_string(),
                        },
                    },
                    FilterTarget::Title {
                        range: StringFilterRange::Equal {
                            value: "Title1".to_string(),
                        },
                    },
                ],
            },
            serde_json::json!({
                "target": "group",
                "op": "or",
                "children": [
                    {
                        "target": "artist",
                        "range": {
                            "op": "equal",
                            "value": "Artist1",
                        }
                    },
                    {
                        "target": "title",
                        "range": {
                            "op": "equal",
                            "value": "Title1",
                        }
                    }
                ]
            }),
        );
    }
}

mod test_filter_target {
    use crate::filter::{
        ArtworkFilterRange, BoolFilterRange, DateFilterRange, FilterTarget, IntFilterRange,
        StringFilterRange, TagsFilterRange,
    };
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn tags() {
        assert_serde(
            FilterTarget::Tags {
                range: TagsFilterRange::Contain { value: 1 },
            },
            serde_json::json!({
                "target": "tags",
                "range": {
                    "op": "contain",
                    "value": 1,
                },
            }),
        );
    }

    #[test]
    fn rating() {
        assert_serde(
            FilterTarget::Rating {
                range: IntFilterRange::Equal { value: 5 },
            },
            serde_json::json!({
                "target": "rating",
                "range": {
                    "op": "equal",
                    "value": 5,
                },
            }),
        );
    }

    #[test]
    fn genre() {
        assert_serde(
            FilterTarget::Genre {
                range: StringFilterRange::Equal {
                    value: "Rock".to_string(),
                },
            },
            serde_json::json!({
                "target": "genre",
                "range": {
                    "op": "equal",
                    "value": "Rock",
                },
            }),
        );
    }

    #[test]
    fn artist() {
        assert_serde(
            FilterTarget::Artist {
                range: StringFilterRange::Equal {
                    value: "Test Artist".to_string(),
                },
            },
            serde_json::json!({
                "target": "artist",
                "range": {
                    "op": "equal",
                    "value": "Test Artist",
                },
            }),
        );
    }

    #[test]
    fn albumartist() {
        assert_serde(
            FilterTarget::Albumartist {
                range: StringFilterRange::Equal {
                    value: "Test Album Artist".to_string(),
                },
            },
            serde_json::json!({
                "target": "albumartist",
                "range": {
                    "op": "equal",
                    "value": "Test Album Artist",
                },
            }),
        );
    }

    #[test]
    fn album() {
        assert_serde(
            FilterTarget::Album {
                range: StringFilterRange::Equal {
                    value: "Test Album".to_string(),
                },
            },
            serde_json::json!({
                "target": "album",
                "range": {
                    "op": "equal",
                    "value": "Test Album",
                },
            }),
        );
    }

    #[test]
    fn composer() {
        assert_serde(
            FilterTarget::Composer {
                range: StringFilterRange::Equal {
                    value: "Test Composer".to_string(),
                },
            },
            serde_json::json!({
                "target": "composer",
                "range": {
                    "op": "equal",
                    "value": "Test Composer",
                },
            }),
        );
    }

    #[test]
    fn title() {
        assert_serde(
            FilterTarget::Title {
                range: StringFilterRange::Equal {
                    value: "Test Title".to_string(),
                },
            },
            serde_json::json!({
                "target": "title",
                "range": {
                    "op": "equal",
                    "value": "Test Title",
                },
            }),
        );
    }

    #[test]
    fn artwork() {
        assert_serde(
            FilterTarget::Artwork {
                range: ArtworkFilterRange::Has,
            },
            serde_json::json!({
                "target": "artwork",
                "range": {
                    "op": "has",
                },
            }),
        );
    }

    #[test]
    fn duration() {
        assert_serde(
            FilterTarget::Duration {
                range: IntFilterRange::Equal { value: 240000 },
            },
            serde_json::json!({
                "target": "duration",
                "range": {
                    "op": "equal",
                    "value": 240000,
                },
            }),
        );
    }

    #[test]
    fn release_date() {
        let date = NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        assert_serde(
            FilterTarget::ReleaseDate {
                range: DateFilterRange::Equal { value: date },
            },
            serde_json::json!({
                "target": "release_date",
                "range": {
                    "op": "equal",
                    "value": date,
                },
            }),
        );
    }

    #[test]
    fn track_number() {
        assert_serde(
            FilterTarget::TrackNumber {
                range: IntFilterRange::Equal { value: 1 },
            },
            serde_json::json!({
                "target": "track_number",
                "range": {
                    "op": "equal",
                    "value": 1,
                },
            }),
        );
    }

    #[test]
    fn track_max() {
        assert_serde(
            FilterTarget::TrackMax {
                range: IntFilterRange::Equal { value: 12 },
            },
            serde_json::json!({
                "target": "track_max",
                "range": {
                    "op": "equal",
                    "value": 12,
                },
            }),
        );
    }

    #[test]
    fn disc_number() {
        assert_serde(
            FilterTarget::DiscNumber {
                range: IntFilterRange::Equal { value: 1 },
            },
            serde_json::json!({
                "target": "disc_number",
                "range": {
                    "op": "equal",
                    "value": 1,
                },
            }),
        );
    }

    #[test]
    fn disc_max() {
        assert_serde(
            FilterTarget::DiscMax {
                range: IntFilterRange::Equal { value: 2 },
            },
            serde_json::json!({
                "target": "disc_max",
                "range": {
                    "op": "equal",
                    "value": 2,
                },
            }),
        );
    }

    #[test]
    fn memo() {
        assert_serde(
            FilterTarget::Memo {
                range: StringFilterRange::Equal {
                    value: "Test memo".to_string(),
                },
            },
            serde_json::json!({
                "target": "memo",
                "range": {
                    "op": "equal",
                    "value": "Test memo",
                },
            }),
        );
    }

    #[test]
    fn memo_manage() {
        assert_serde(
            FilterTarget::MemoManage {
                range: StringFilterRange::Equal {
                    value: "Test manage memo".to_string(),
                },
            },
            serde_json::json!({
                "target": "memo_manage",
                "range": {
                    "op": "equal",
                    "value": "Test manage memo",
                },
            }),
        );
    }

    #[test]
    fn entry_date() {
        let date = NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        assert_serde(
            FilterTarget::EntryDate {
                range: DateFilterRange::Equal { value: date },
            },
            serde_json::json!({
                "target": "entry_date",
                "range": {
                    "op": "equal",
                    "value": date,
                },
            }),
        );
    }

    #[test]
    fn original_track() {
        assert_serde(
            FilterTarget::OriginalTrack {
                range: StringFilterRange::Equal {
                    value: "Original Track".to_string(),
                },
            },
            serde_json::json!({
                "target": "original_track",
                "range": {
                    "op": "equal",
                    "value": "Original Track",
                },
            }),
        );
    }

    #[test]
    fn suggest_target() {
        assert_serde(
            FilterTarget::SuggestTarget {
                range: BoolFilterRange::True,
            },
            serde_json::json!({
                "target": "suggest_target",
                "range": {
                    "op": "true",
                },
            }),
        );
    }
}

mod test_artwork_range {
    use crate::filter::ArtworkFilterRange;

    use super::*;

    #[test]
    fn has() {
        assert_serde(
            ArtworkFilterRange::Has,
            serde_json::json!({
                "op": "has",
            }),
        );
    }

    #[test]
    fn none() {
        assert_serde(
            ArtworkFilterRange::None,
            serde_json::json!({
                "op": "none",
            }),
        );
    }
}

mod test_bool_range {
    use crate::filter::BoolFilterRange;

    use super::*;

    #[test]
    fn true_value() {
        assert_serde(
            BoolFilterRange::True,
            serde_json::json!({
                "op": "true",
            }),
        );
    }

    #[test]
    fn false_value() {
        assert_serde(
            BoolFilterRange::False,
            serde_json::json!({
                "op": "false",
            }),
        );
    }
}

mod test_date_range {
    use crate::filter::DateFilterRange;
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn equal() {
        let date = NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        assert_serde(
            DateFilterRange::Equal { value: date },
            serde_json::json!({
                "op": "equal",
                "value": date,
            }),
        );
    }

    #[test]
    fn not_equal() {
        let date = NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        assert_serde(
            DateFilterRange::NotEqual { value: date },
            serde_json::json!({
                "op": "not_equal",
                "value": date,
            }),
        );
    }

    #[test]
    fn before() {
        let date = NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        assert_serde(
            DateFilterRange::Before { value: date },
            serde_json::json!({
                "op": "before",
                "value": date,
            }),
        );
    }

    #[test]
    fn after() {
        let date = NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        assert_serde(
            DateFilterRange::After { value: date },
            serde_json::json!({
                "op": "after",
                "value": date,
            }),
        );
    }

    #[test]
    fn none() {
        assert_serde(
            DateFilterRange::None,
            serde_json::json!({
                "op": "none",
            }),
        );
    }
}

mod test_int_range {
    use crate::filter::IntFilterRange;

    use super::*;

    #[test]
    fn equal() {
        assert_serde(
            IntFilterRange::Equal { value: 42 },
            serde_json::json!({
                "op": "equal",
                "value": 42,
            }),
        );
    }

    #[test]
    fn not_equal() {
        assert_serde(
            IntFilterRange::NotEqual { value: 42 },
            serde_json::json!({
                "op": "not_equal",
                "value": 42,
            }),
        );
    }

    #[test]
    fn large_equal() {
        assert_serde(
            IntFilterRange::LargeEqual { value: 42 },
            serde_json::json!({
                "op": "large_equal",
                "value": 42,
            }),
        );
    }

    #[test]
    fn small_equal() {
        assert_serde(
            IntFilterRange::SmallEqual { value: 42 },
            serde_json::json!({
                "op": "small_equal",
                "value": 42,
            }),
        );
    }

    #[test]
    fn range_in() {
        assert_serde(
            IntFilterRange::RangeIn { min: 10, max: 50 },
            serde_json::json!({
                "op": "range_in",
                "min": 10,
                "max": 50,
            }),
        );
    }

    #[test]
    fn range_out() {
        assert_serde(
            IntFilterRange::RangeOut { min: 10, max: 50 },
            serde_json::json!({
                "op": "range_out",
                "min": 10,
                "max": 50,
            }),
        );
    }
}

mod test_string_range {
    use crate::filter::StringFilterRange;

    use super::*;

    #[test]
    fn equal() {
        assert_serde(
            StringFilterRange::Equal {
                value: "test".to_string(),
            },
            serde_json::json!({
                "op": "equal",
                "value": "test",
            }),
        );
    }

    #[test]
    fn not_equal() {
        assert_serde(
            StringFilterRange::NotEqual {
                value: "test".to_string(),
            },
            serde_json::json!({
                "op": "not_equal",
                "value": "test",
            }),
        );
    }

    #[test]
    fn contain() {
        assert_serde(
            StringFilterRange::Contain {
                value: "test".to_string(),
            },
            serde_json::json!({
                "op": "contain",
                "value": "test",
            }),
        );
    }

    #[test]
    fn not_contain() {
        assert_serde(
            StringFilterRange::NotContain {
                value: "test".to_string(),
            },
            serde_json::json!({
                "op": "not_contain",
                "value": "test",
            }),
        );
    }

    #[test]
    fn start() {
        assert_serde(
            StringFilterRange::Start {
                value: "test".to_string(),
            },
            serde_json::json!({
                "op": "start",
                "value": "test",
            }),
        );
    }

    #[test]
    fn end() {
        assert_serde(
            StringFilterRange::End {
                value: "test".to_string(),
            },
            serde_json::json!({
                "op": "end",
                "value": "test",
            }),
        );
    }
}

mod test_tags_range {
    use crate::filter::TagsFilterRange;

    use super::*;

    #[test]
    fn tags_contain() {
        assert_serde(
            TagsFilterRange::Contain { value: 1 },
            serde_json::json!({
                "op": "contain",
                "value": 1,
            }),
        );
    }

    #[test]
    fn tags_not_contain() {
        assert_serde(
            TagsFilterRange::NotContain { value: 1 },
            serde_json::json!({
                "op": "not_contain",
                "value": 1,
            }),
        );
    }

    #[test]
    fn tags_none() {
        assert_serde(
            TagsFilterRange::None,
            serde_json::json!({
                "op": "none",
            }),
        );
    }
}
