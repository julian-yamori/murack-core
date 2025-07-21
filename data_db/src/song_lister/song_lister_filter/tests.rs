//! SongListerFilterのテスト

use chrono::NaiveDate;
use domain::test_utils::assert_eq_not_orderd;
use sqlx::PgPool;
use test_case::test_case;

use super::*;
use crate::{
    artwork::{SongArtworkDao, SongArtworkDaoImpl},
    song::{SongDao, SongDaoImpl, SongEntry},
    tag::{SongTagsDao, SongTagsDaoImpl},
};

fn target() -> SongListerFilterImpl {
    SongListerFilterImpl {}
}

struct TestDb {
    tx: DbTransaction<'static>,
    song_dao: SongDaoImpl,
    song_tags_dao: SongTagsDaoImpl,
    song_artwork_dao: SongArtworkDaoImpl,
}
impl TestDb {
    async fn new(db_pool: &PgPool) -> Self {
        let tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await.unwrap(),
        };

        Self {
            tx,
            song_dao: SongDaoImpl {},
            song_tags_dao: SongTagsDaoImpl {},
            song_artwork_dao: SongArtworkDaoImpl {},
        }
    }
}
impl TestDb {
    async fn insert_song(&mut self, entry: &SongEntry<'_>) -> i32 {
        self.song_dao.insert(&mut self.tx, entry).await.unwrap()
    }
    async fn insert_tag(&mut self, song_id: i32, tag_id: i32) {
        self.song_tags_dao
            .insert(&mut self.tx, song_id, tag_id)
            .await
            .unwrap()
    }
}

fn dummy_song() -> SongEntry<'static> {
    SongEntry {
        duration: 0,
        path: "",
        folder_id: None,
        title: "",
        artist: "",
        album_artist: "",
        album: "",
        composer: "",
        genre: "",
        track_number: None,
        track_max: None,
        disc_number: None,
        disc_max: None,
        release_date: None,
        memo: "",
        rating: 0,
        original_song: "",
        memo_manage: "",
        suggest_target: false,
        lyrics: "",
        title_order: "",
        artist_order: "",
        album_artist_order: "",
        album_order: "",
        genre_order: "",
        composer_order: "",
    }
}

#[sqlx::test(migrator = "crate::MIGRATOR")]
fn test_group(db_pool: PgPool) {
    async fn insert_song(
        test_db: &mut TestDb,
        artist: &str,
        tags: &[i32],
        rating: i16,
        release_date: Option<NaiveDate>,
    ) -> i32 {
        let mut song = dummy_song();
        song.artist = artist;
        song.rating = rating;
        song.release_date = release_date;

        let song_id = test_db.insert_song(&song).await;
        for tag_id in tags {
            test_db.insert_tag(song_id, *tag_id).await;
        }

        song_id
    }

    let mut test_db = TestDb::new(&db_pool).await;

    let hit_1 = insert_song(&mut test_db, "taro", &[45, 58], 3, None).await;
    insert_song(
        &mut test_db,
        "jiro",
        &[45, 58],
        4,
        Some(NaiveDate::from_ymd_opt(2021, 9, 25).unwrap()),
    )
    .await;
    let hit_2 = insert_song(
        &mut test_db,
        "taro",
        &[],
        5,
        Some(NaiveDate::from_ymd_opt(1999, 9, 9).unwrap()),
    )
    .await;
    insert_song(&mut test_db, "taro", &[8, 9, 10], 0, None).await;
    insert_song(
        &mut test_db,
        "3bro",
        &[],
        2,
        Some(NaiveDate::from_ymd_opt(1999, 9, 9).unwrap()),
    )
    .await;
    let hit_3 = insert_song(
        &mut test_db,
        "taro",
        &[999],
        0,
        Some(NaiveDate::from_ymd_opt(2021, 9, 25).unwrap()),
    )
    .await;
    let hit_4 = insert_song(
        &mut test_db,
        "taro",
        &[8, 9, 10],
        4,
        Some(NaiveDate::from_ymd_opt(2021, 9, 25).unwrap()),
    )
    .await;

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

    let target = target();
    assert_eq_not_orderd(
        &target.list_song_id(&mut test_db.tx, &filter).await.unwrap(),
        &[hit_1, hit_2, hit_3, hit_4],
    )
}

#[sqlx::test(migrator = "crate::MIGRATOR")]
fn test_str(db_pool: PgPool) {
    async fn insert_song(test_db: &mut TestDb, artist: &str) -> i32 {
        let mut song = dummy_song();
        song.artist = artist;
        test_db.insert_song(&song).await
    }

    fn filter(range: StringFilterRange) -> FilterTarget {
        FilterTarget::Artist { range }
    }

    let mut test_db = TestDb::new(&db_pool).await;

    let song_1 = insert_song(&mut test_db, "test").await;
    let song_2 = insert_song(&mut test_db, "AAtest").await;
    let song_3 = insert_song(&mut test_db, "testAA").await;
    let song_4 = insert_song(&mut test_db, "AAtestAA").await;
    let song_5 = insert_song(&mut test_db, "testAAtestAAtestAAtest").await;
    let song_6 = insert_song(&mut test_db, "teAAst").await;
    let song_7 = insert_song(&mut test_db, "AAAAAA").await;
    let song_8 = insert_song(&mut test_db, "").await;
    let song_9 = insert_song(&mut test_db, "te%st").await;
    let song_10 = insert_song(&mut test_db, "AAte%stAA").await;

    let target = target();
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(StringFilterRange::Equal {
                    value: "test".to_owned(),
                }),
            )
            .await
            .unwrap(),
        &[song_1],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(StringFilterRange::NotEqual {
                    value: "test".to_owned(),
                }),
            )
            .await
            .unwrap(),
        &[
            song_2, song_3, song_4, song_5, song_6, song_7, song_8, song_9, song_10,
        ],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(StringFilterRange::Start {
                    value: "test".to_owned(),
                }),
            )
            .await
            .unwrap(),
        &[song_1, song_3, song_5],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(StringFilterRange::End {
                    value: "test".to_owned(),
                }),
            )
            .await
            .unwrap(),
        &[song_1, song_2, song_5],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(StringFilterRange::Contain {
                    value: "test".to_owned(),
                }),
            )
            .await
            .unwrap(),
        &[song_1, song_2, song_3, song_4, song_5],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(StringFilterRange::NotContain {
                    value: "test".to_owned(),
                }),
            )
            .await
            .unwrap(),
        &[song_6, song_7, song_8, song_9, song_10],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(StringFilterRange::Equal {
                    value: "te%st".to_owned(),
                }),
            )
            .await
            .unwrap(),
        &[song_9],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(StringFilterRange::Contain {
                    value: "te%st".to_owned(),
                }),
            )
            .await
            .unwrap(),
        &[song_9, song_10],
    );
}

#[cfg(test)]
mod test_int {
    use super::*;

    struct DbFixture {
        test_db: TestDb,

        // track_number 別の id リスト
        #[allow(dead_code)]
        song_none: i32,
        song_1: i32,
        song_5: i32,
        song_9: i32,
        song_10: i32,
        song_25: i32,
        song_123: i32,
    }
    impl DbFixture {
        async fn new(db_pool: &PgPool) -> Self {
            async fn insert_song(test_db: &mut TestDb, track_number: Option<i32>) -> i32 {
                let mut song = dummy_song();
                song.track_number = track_number;
                test_db.insert_song(&song).await
            }

            let mut test_db = TestDb::new(db_pool).await;

            Self {
                song_none: insert_song(&mut test_db, None).await,
                song_1: insert_song(&mut test_db, Some(1)).await,
                song_5: insert_song(&mut test_db, Some(5)).await,
                song_9: insert_song(&mut test_db, Some(9)).await,
                song_10: insert_song(&mut test_db, Some(10)).await,
                song_25: insert_song(&mut test_db, Some(25)).await,
                song_123: insert_song(&mut test_db, Some(123)).await,
                test_db,
            }
        }
    }

    fn filter(range: IntFilterRange) -> FilterTarget {
        FilterTarget::TrackNumber { range }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_equal(db_pool: PgPool) {
        let DbFixture {
            mut test_db,
            song_none: _,
            song_1: _,
            song_5: _,
            song_9,
            song_10: _,
            song_25: _,
            song_123: _,
        } = DbFixture::new(&db_pool).await;

        let target = target();
        assert_eq_not_orderd(
            &target
                .list_song_id(&mut test_db.tx, &filter(IntFilterRange::Equal { value: 9 }))
                .await
                .unwrap(),
            &[song_9],
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_not_equal(db_pool: PgPool) {
        let DbFixture {
            mut test_db,
            song_none: _,
            song_1,
            song_5,
            song_9,
            song_10,
            song_25: _,
            song_123,
        } = DbFixture::new(&db_pool).await;

        let target = target();
        //※nullは含めない仕様(WalkBase1がそうなっていたので)
        assert_eq_not_orderd(
            &target
                .list_song_id(
                    &mut test_db.tx,
                    &filter(IntFilterRange::NotEqual { value: 25 }),
                )
                .await
                .unwrap(),
            &[song_1, song_5, song_9, song_10, song_123],
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_large_equal(db_pool: PgPool) {
        let DbFixture {
            mut test_db,
            song_none: _,
            song_1: _,
            song_5: _,
            song_9: _,
            song_10,
            song_25,
            song_123,
        } = DbFixture::new(&db_pool).await;

        let target = target();
        assert_eq_not_orderd(
            &target
                .list_song_id(
                    &mut test_db.tx,
                    &filter(IntFilterRange::LargeEqual { value: 10 }),
                )
                .await
                .unwrap(),
            &[song_10, song_25, song_123],
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_amall_equal(db_pool: PgPool) {
        let DbFixture {
            mut test_db,
            song_none: _,
            song_1,
            song_5,
            song_9: _,
            song_10: _,
            song_25: _,
            song_123: _,
        } = DbFixture::new(&db_pool).await;

        let target = target();
        assert_eq_not_orderd(
            &target
                .list_song_id(
                    &mut test_db.tx,
                    &filter(IntFilterRange::SmallEqual { value: 5 }),
                )
                .await
                .unwrap(),
            &[song_1, song_5],
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_range_in(db_pool: PgPool) {
        let DbFixture {
            mut test_db,
            song_none: _,
            song_1: _,
            song_5: _,
            song_9,
            song_10,
            song_25,
            song_123: _,
        } = DbFixture::new(&db_pool).await;

        let target = target();
        assert_eq_not_orderd(
            &target
                .list_song_id(
                    &mut test_db.tx,
                    &filter(IntFilterRange::RangeIn { min: 9, max: 25 }),
                )
                .await
                .unwrap(),
            &[song_9, song_10, song_25],
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    fn test_range_out(db_pool: PgPool) {
        let DbFixture {
            mut test_db,
            song_none: _,
            song_1,
            song_5: _,
            song_9: _,
            song_10: _,
            song_25,
            song_123,
        } = DbFixture::new(&db_pool).await;

        let target = target();
        assert_eq_not_orderd(
            &target
                .list_song_id(
                    &mut test_db.tx,
                    &filter(IntFilterRange::RangeOut { min: 5, max: 10 }),
                )
                .await
                .unwrap(),
            &[song_1, song_25, song_123],
        );
    }

    // #[sqlx::test(migrator = "crate::MIGRATOR")]
    // fn test_equal_none(db_pool: PgPool) {
    //     let DbFixture {
    //         mut test_db,
    //         song_none,
    //         song_1,
    //         song_5,
    //         song_9,
    //         song_10,
    //         song_25,
    //         song_123,
    //     } = DbFixture::new(&db_pool).await;

    //     let target = target();
    //     assert_eq_not_orderd(
    //         &target
    //             .list_song_id(
    //                 &mut test_db.tx,
    //                 &filter(IntFilterRange::Equal { value: None }),
    //             )
    //             .await
    //             .unwrap(),
    //         &[song_none],
    //     );
    // }

    // #[sqlx::test(migrator = "crate::MIGRATOR")]
    // fn test_not_equal_none(db_pool: PgPool) {
    //     let DbFixture {
    //         mut test_db,
    //         song_none: _,
    //         song_1,
    //         song_5,
    //         song_9,
    //         song_10,
    //         song_25,
    //         song_123,
    //     } = DbFixture::new(&db_pool).await;

    //     let target = target();
    //     assert_eq_not_orderd(
    //         &target
    //             .list_song_id(
    //                 &mut test_db.tx,
    //                 &filter(IntFilterRange::NotEqual { value: None }),
    //             )
    //             .await
    //             .unwrap(),
    //         &[song_1, song_5, song_9, song_10, song_25, song_123],
    //     );
    // }

    // #[sqlx::test(migrator = "crate::MIGRATOR")]
    // fn test_large_equal_none(db_pool: PgPool) {
    //     let DbFixture {
    //         mut test_db,
    //         song_none: _,
    //         song_1,
    //         song_5,
    //         song_9,
    //         song_10,
    //         song_25,
    //         song_123,
    //     } = DbFixture::new(&db_pool).await;

    //     let target = target();
    //     assert_eq_not_orderd(
    //         &target
    //             .list_song_id(
    //                 &mut test_db.tx,
    //                 &filter(IntFilterRange::LargeEqual { value: None }),
    //             )
    //             .await
    //             .unwrap(),
    //         &[],
    //     );
    // }

    // #[sqlx::test(migrator = "crate::MIGRATOR")]
    // fn test_range_min_is_none(db_pool: PgPool) {
    //     let DbFixture {
    //         mut test_db,
    //         song_none: _,
    //         song_1,
    //         song_5,
    //         song_9,
    //         song_10,
    //         song_25,
    //         song_123,
    //     } = DbFixture::new(&db_pool).await;

    //     let target = target();
    //     assert_eq_not_orderd(
    //         &target
    //             .list_song_id(
    //                 &mut test_db.tx,
    //                 &filter(IntFilterRange::RangeIn { min: None, max: 5 }),
    //             )
    //             .await
    //             .unwrap(),
    //         &[],
    //     );
    // }

    // #[sqlx::test(migrator = "crate::MIGRATOR")]
    // fn test_range_max_is_none(db_pool: PgPool) {
    //     let DbFixture {
    //         mut test_db,
    //         song_none: _,
    //         song_1,
    //         song_5,
    //         song_9,
    //         song_10,
    //         song_25,
    //         song_123,
    //     } = DbFixture::new(&db_pool).await;

    //     let target = target();
    //     assert_eq_not_orderd(
    //         &target
    //             .list_song_id(
    //                 &mut test_db.tx,
    //                 &filter(IntFilterRange::RangeIn { min: 5, max: None }),
    //             )
    //             .await
    //             .unwrap(),
    //         &[],
    //     );
    // }
}

#[sqlx::test(migrator = "crate::MIGRATOR")]
fn test_tag(db_pool: PgPool) {
    async fn insert_song(test_db: &mut TestDb, tags: &[i32]) -> i32 {
        let song_id = test_db.insert_song(&dummy_song()).await;
        for tag_id in tags {
            test_db.insert_tag(song_id, *tag_id).await;
        }

        song_id
    }

    fn filter(range: TagsFilterRange) -> FilterTarget {
        FilterTarget::Tags { range }
    }

    let mut test_db = TestDb::new(&db_pool).await;

    let song_0 = insert_song(&mut test_db, &[]).await;
    let song_1 = insert_song(&mut test_db, &[4]).await;
    let song_2 = insert_song(&mut test_db, &[4, 83]).await;
    let song_3 = insert_song(&mut test_db, &[8, 83]).await;

    let target = target();
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(TagsFilterRange::Contain { value: 4 }),
            )
            .await
            .unwrap(),
        &[song_1, song_2],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(TagsFilterRange::NotContain { value: 4 }),
            )
            .await
            .unwrap(),
        &[song_0, song_3],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(TagsFilterRange::Contain { value: 5 }),
            )
            .await
            .unwrap(),
        &[],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(TagsFilterRange::NotContain { value: 5 }),
            )
            .await
            .unwrap(),
        &[song_0, song_1, song_2, song_3],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(TagsFilterRange::Contain { value: 83 }),
            )
            .await
            .unwrap(),
        &[song_2, song_3],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(TagsFilterRange::NotContain { value: 83 }),
            )
            .await
            .unwrap(),
        &[song_0, song_1],
    );

    // assert_eq_not_orderd(
    //     &target
    //         .list_song_id(
    //             &mut test_db.tx,
    //             &filter(TagsFilterRange::Contain { value: None }),
    //         )
    //         .await
    //         .unwrap(),
    //     &[],
    // );
    // assert_eq_not_orderd(
    //     &target
    //         .list_song_id(
    //             &mut test_db.tx,
    //             &filter(TagsFilterRange::NotContain { value: None }),
    //         )
    //         .await
    //         .unwrap(),
    //     &[],
    // );
    assert_eq_not_orderd(
        &target
            .list_song_id(&mut test_db.tx, &filter(TagsFilterRange::None))
            .await
            .unwrap(),
        &[song_0],
    );
}

#[sqlx::test(migrator = "crate::MIGRATOR")]
fn test_bool(db_pool: PgPool) {
    async fn insert_song(test_db: &mut TestDb, suggest_target: bool) -> i32 {
        let mut song = dummy_song();
        song.suggest_target = suggest_target;
        test_db.insert_song(&song).await
    }

    fn filter(range: BoolFilterRange) -> FilterTarget {
        FilterTarget::SuggestTarget { range }
    }

    let mut test_db = TestDb::new(&db_pool).await;

    let song_true = insert_song(&mut test_db, true).await;
    let song_false = insert_song(&mut test_db, false).await;

    let target = target();
    assert_eq_not_orderd(
        &target
            .list_song_id(&mut test_db.tx, &filter(BoolFilterRange::True))
            .await
            .unwrap(),
        &[song_true],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(&mut test_db.tx, &filter(BoolFilterRange::False))
            .await
            .unwrap(),
        &[song_false],
    );
}

#[sqlx::test(migrator = "crate::MIGRATOR")]
fn test_artwork(db_pool: PgPool) {
    async fn insert_song(test_db: &mut TestDb, artworks: &[i32]) -> i32 {
        let song_id = test_db.insert_song(&dummy_song()).await;
        for (idx, artwork_id) in artworks.iter().enumerate() {
            test_db
                .song_artwork_dao
                .insert(
                    &mut test_db.tx,
                    song_id,
                    idx,
                    *artwork_id,
                    3 + (idx as u8),
                    "",
                )
                .await
                .unwrap();
        }

        song_id
    }
    fn filter(range: ArtworkFilterRange) -> FilterTarget {
        FilterTarget::Artwork { range }
    }

    let mut test_db = TestDb::new(&db_pool).await;

    let song_0 = insert_song(&mut test_db, &[]).await;
    let song_1 = insert_song(&mut test_db, &[7]).await;
    let song_2 = insert_song(&mut test_db, &[5, 6]).await;

    let target = target();
    assert_eq_not_orderd(
        &target
            .list_song_id(&mut test_db.tx, &filter(ArtworkFilterRange::Has))
            .await
            .unwrap(),
        &[song_1, song_2],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(&mut test_db.tx, &filter(ArtworkFilterRange::None))
            .await
            .unwrap(),
        &[song_0],
    );
}

#[sqlx::test(migrator = "crate::MIGRATOR")]
fn test_date(db_pool: PgPool) {
    async fn insert_song(test_db: &mut TestDb, release_date: Option<NaiveDate>) -> i32 {
        let mut song = dummy_song();
        song.release_date = release_date;
        test_db.insert_song(&song).await
    }

    fn filter(range: DateFilterRange) -> FilterTarget {
        FilterTarget::ReleaseDate { range }
    }

    let mut test_db = TestDb::new(&db_pool).await;

    let song_0 = insert_song(&mut test_db, None).await;
    let song_1 = insert_song(
        &mut test_db,
        Some(NaiveDate::from_ymd_opt(1998, 12, 10).unwrap()),
    )
    .await;
    let song_2 = insert_song(
        &mut test_db,
        Some(NaiveDate::from_ymd_opt(2012, 4, 5).unwrap()),
    )
    .await;
    let song_3 = insert_song(
        &mut test_db,
        Some(NaiveDate::from_ymd_opt(2021, 9, 26).unwrap()),
    )
    .await;

    let target = target();
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(DateFilterRange::Equal {
                    value: NaiveDate::from_ymd_opt(2012, 4, 5).unwrap(),
                }),
            )
            .await
            .unwrap(),
        &[song_2],
    );
    //※nullは含めない仕様(WalkBase1がそうなっていたので)
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(DateFilterRange::NotEqual {
                    value: NaiveDate::from_ymd_opt(2012, 4, 5).unwrap(),
                }),
            )
            .await
            .unwrap(),
        &[song_1, song_3],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(DateFilterRange::Before {
                    value: NaiveDate::from_ymd_opt(2012, 11, 12).unwrap(),
                }),
            )
            .await
            .unwrap(),
        &[song_1, song_2],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(
                &mut test_db.tx,
                &filter(DateFilterRange::After {
                    value: NaiveDate::from_ymd_opt(2012, 4, 5).unwrap(),
                }),
            )
            .await
            .unwrap(),
        &[song_2, song_3],
    );
    assert_eq_not_orderd(
        &target
            .list_song_id(&mut test_db.tx, &filter(DateFilterRange::None))
            .await
            .unwrap(),
        &[song_0],
    );

    // assert_eq_not_orderd(
    //     &target
    //         .list_song_id(
    //             &mut test_db.tx,
    //             &filter(DateFilterRange::Equal { value: None }),
    //         )
    //         .await
    //         .unwrap(),
    //     &[song_0],
    // );
    // assert_eq_not_orderd(
    //     &target
    //         .list_song_id(
    //             &mut test_db.tx,
    //             &filter(DateFilterRange::NotEqual { value: None }),
    //         )
    //         .await
    //         .unwrap(),
    //     &[song_1, song_2, song_3],
    // );
    // assert_eq_not_orderd(
    //     &target
    //         .list_song_id(
    //             &mut test_db.tx,
    //             &filter(DateFilterRange::Before { value: None }),
    //         )
    //         .await
    //         .unwrap(),
    //     &[],
    // );
    // assert_eq_not_orderd(
    //     &target
    //         .list_song_id(
    //             &mut test_db.tx,
    //             &filter(DateFilterRange::After { value: None }),
    //         )
    //         .await
    //         .unwrap(),
    //     &[],
    // );
}

#[test_case(15, 28, 15, 28 ; "normal")]
#[test_case(28, 15, 15, 28 ; "inversed")]
#[test_case(6, 113, 6, 113 ; "digit_dif")]
#[test_case(113, 6, 6, 113 ; "digit_dif_inversed")]
fn test_get_ordered_int(value1: i32, value2: i32, expect_1: i32, expect_2: i32) {
    let result = get_ordered_int(value1, value2);
    assert_eq!(result.0, expect_1);
    assert_eq!(result.1, expect_2);
}
