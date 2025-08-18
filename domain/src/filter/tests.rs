//! TrackListerFilterのテスト

use sqlx::{PgPool, PgTransaction};

use super::*;
use crate::test_utils::assert_eq_not_orderd;

/// フィルタを使用して曲 ID を列挙
async fn get_track_ids<'c>(
    tx: &mut PgTransaction<'c>,
    filter: &RootFilter,
) -> sqlx::Result<Vec<i32>> {
    let mut query_base = "SELECT tracks.id FROM tracks".to_owned();

    //フィルタから条件を取得して追加
    if let Some(query_where) = filter.where_expression() {
        query_base = format!("{query_base} WHERE {query_where}");
    }

    let list = sqlx::query_scalar(&query_base).fetch_all(&mut **tx).await?;

    Ok(list)
}

// グループフィルタ（AND/OR組み合わせ）のテスト
mod test_group_filter {
    use super::*;
    use chrono::NaiveDate;

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("group_filter"))]
    async fn complex_and_or_combination(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let filter = FilterTarget::FilterGroup {
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
                            range: DateFilterRange::Equal {
                                value: NaiveDate::from_ymd_opt(2021, 9, 25).unwrap(),
                            },
                        },
                    ],
                },
            ],
        };

        let result = get_track_ids(&mut tx, &filter).await?;

        // track_ids: 1, 3, 6, 7 が該当するはず
        assert_eq_not_orderd(&result, &[1, 3, 6, 7]);

        Ok(())
    }
}

// 文字列フィルタのテスト
mod test_string_filter {
    use super::*;

    fn filter(range: StringFilterRange) -> FilterTarget {
        FilterTarget::Artist { range }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("string_filter"))]
    async fn equal(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(StringFilterRange::Equal {
                value: "test".to_owned(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[1]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("string_filter"))]
    async fn not_equal(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(StringFilterRange::NotEqual {
                value: "test".to_owned(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[2, 3, 4, 5, 6, 7, 8, 9, 10]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("string_filter"))]
    async fn start(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(StringFilterRange::Start {
                value: "test".to_owned(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[1, 3, 5]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("string_filter"))]
    async fn end(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(StringFilterRange::End {
                value: "test".to_owned(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[1, 2, 5]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("string_filter"))]
    async fn contain(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(StringFilterRange::Contain {
                value: "test".to_owned(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[1, 2, 3, 4, 5]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("string_filter"))]
    async fn not_contain(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(StringFilterRange::NotContain {
                value: "test".to_owned(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[6, 7, 8, 9, 10]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("string_filter"))]
    async fn equal_with_special_chars(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(StringFilterRange::Equal {
                value: "te%st".to_owned(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[9]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("string_filter"))]
    async fn contain_with_special_chars(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(StringFilterRange::Contain {
                value: "te%st".to_owned(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[9, 10]);
        Ok(())
    }
}

// 整数フィルタのテスト
mod test_int_filter {
    use super::*;

    // フィクスチャーでは以下の値が設定される:
    // (track id: track_number)
    // - 1: NULL
    // - 2: 1
    // - 3: 5
    // - 4: 9
    // - 5: 10
    // - 6: 25
    // - 7: 123

    fn filter(range: IntFilterRange) -> FilterTarget {
        FilterTarget::TrackNumber { range }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("int_filter"))]
    async fn equal(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(&mut tx, &filter(IntFilterRange::Equal { value: 9 })).await?;

        assert_eq_not_orderd(&result, &[4]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("int_filter"))]
    async fn not_equal(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        // ※nullは含めない仕様(WalkBase1がそうなっていたので)
        let result =
            get_track_ids(&mut tx, &filter(IntFilterRange::NotEqual { value: 25 })).await?;

        assert_eq_not_orderd(&result, &[2, 3, 4, 5, 7]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("int_filter"))]
    async fn large_equal(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result =
            get_track_ids(&mut tx, &filter(IntFilterRange::LargeEqual { value: 10 })).await?;

        assert_eq_not_orderd(&result, &[5, 6, 7]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("int_filter"))]
    async fn small_equal(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result =
            get_track_ids(&mut tx, &filter(IntFilterRange::SmallEqual { value: 5 })).await?;

        assert_eq_not_orderd(&result, &[2, 3]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("int_filter"))]
    async fn range_in(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(IntFilterRange::RangeIn { min: 9, max: 25 }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[4, 5, 6]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("int_filter"))]
    async fn range_out(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(IntFilterRange::RangeOut { min: 5, max: 10 }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[2, 6, 7]);
        Ok(())
    }
}

// タグフィルタのテスト
mod test_tags_filter {
    use super::*;

    fn filter(range: TagsFilterRange) -> FilterTarget {
        FilterTarget::Tags { range }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("tags_filter"))]
    async fn contain_existing_tag(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(&mut tx, &filter(TagsFilterRange::Contain { value: 4 })).await?;

        assert_eq_not_orderd(&result, &[2, 3]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("tags_filter"))]
    async fn not_contain_existing_tag(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result =
            get_track_ids(&mut tx, &filter(TagsFilterRange::NotContain { value: 4 })).await?;

        assert_eq_not_orderd(&result, &[1, 4]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("tags_filter"))]
    async fn contain_nonexistent_tag(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(&mut tx, &filter(TagsFilterRange::Contain { value: 5 })).await?;

        assert_eq_not_orderd(&result, &[]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("tags_filter"))]
    async fn not_contain_nonexistent_tag(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result =
            get_track_ids(&mut tx, &filter(TagsFilterRange::NotContain { value: 5 })).await?;

        assert_eq_not_orderd(&result, &[1, 2, 3, 4]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("tags_filter"))]
    async fn contain_another_tag(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result =
            get_track_ids(&mut tx, &filter(TagsFilterRange::Contain { value: 83 })).await?;

        assert_eq_not_orderd(&result, &[3, 4]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("tags_filter"))]
    async fn not_contain_another_tag(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result =
            get_track_ids(&mut tx, &filter(TagsFilterRange::NotContain { value: 83 })).await?;

        assert_eq_not_orderd(&result, &[1, 2]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("tags_filter"))]
    async fn none(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(&mut tx, &filter(TagsFilterRange::None)).await?;

        assert_eq_not_orderd(&result, &[1]);
        Ok(())
    }
}

// ブールフィルタのテスト
mod test_bool_filter {
    use super::*;

    fn filter(range: BoolFilterRange) -> FilterTarget {
        FilterTarget::SuggestTarget { range }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("bool_filter"))]
    async fn is_true(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(&mut tx, &filter(BoolFilterRange::True)).await?;

        assert_eq_not_orderd(&result, &[1]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("bool_filter"))]
    async fn is_false(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(&mut tx, &filter(BoolFilterRange::False)).await?;

        assert_eq_not_orderd(&result, &[2]);
        Ok(())
    }
}

// アートワークフィルタのテスト
mod test_artwork_filter {
    use super::*;

    fn filter(range: ArtworkFilterRange) -> FilterTarget {
        FilterTarget::Artwork { range }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("artwork_filter"))]
    async fn has_artwork(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(&mut tx, &filter(ArtworkFilterRange::Has)).await?;

        assert_eq_not_orderd(&result, &[2, 3]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("artwork_filter"))]
    async fn no_artwork(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(&mut tx, &filter(ArtworkFilterRange::None)).await?;

        assert_eq_not_orderd(&result, &[1]);
        Ok(())
    }
}

// 日付フィルタのテスト
mod test_date_filter {
    use super::*;
    use chrono::NaiveDate;

    fn filter(range: DateFilterRange) -> FilterTarget {
        FilterTarget::ReleaseDate { range }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("date_filter"))]
    async fn equal(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(DateFilterRange::Equal {
                value: NaiveDate::from_ymd_opt(2012, 4, 5).unwrap(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[3]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("date_filter"))]
    async fn not_equal(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        // ※nullは含めない仕様(WalkBase1がそうなっていたので)
        let result = get_track_ids(
            &mut tx,
            &filter(DateFilterRange::NotEqual {
                value: NaiveDate::from_ymd_opt(2012, 4, 5).unwrap(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[2, 4]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("date_filter"))]
    async fn before(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(DateFilterRange::Before {
                value: NaiveDate::from_ymd_opt(2012, 11, 12).unwrap(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[2, 3]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("date_filter"))]
    async fn after(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(
            &mut tx,
            &filter(DateFilterRange::After {
                value: NaiveDate::from_ymd_opt(2012, 4, 5).unwrap(),
            }),
        )
        .await?;

        assert_eq_not_orderd(&result, &[3, 4]);
        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR", fixtures("date_filter"))]
    async fn none(pool: PgPool) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        let result = get_track_ids(&mut tx, &filter(DateFilterRange::None)).await?;

        assert_eq_not_orderd(&result, &[1]);
        Ok(())
    }
}

// get_ordered_int ユーティリティ関数のテスト
mod test_utility {
    use test_case::test_case;

    #[test_case(15, 28, 15, 28 ; "normal")]
    #[test_case(28, 15, 15, 28 ; "inversed")]
    #[test_case(6, 113, 6, 113 ; "digit_dif")]
    #[test_case(113, 6, 6, 113 ; "digit_dif_inversed")]
    fn test_get_ordered_int(value1: i32, value2: i32, expect_1: i32, expect_2: i32) {
        let result = super::get_ordered_int(value1, value2);
        assert_eq!(result.0, expect_1);
        assert_eq!(result.1, expect_2);
    }
}
