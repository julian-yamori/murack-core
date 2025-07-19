//! SongListerFilterのテスト

use chrono::NaiveDate;
use domain::{folder::FolderIdMayRoot, mocks, path::LibSongPath, test_utils::assert_eq_not_orderd};
use once_cell::sync::Lazy;
use paste::paste;
use sqlx::PgPool;
use test_case::test_case;

use super::*;
use crate::{
    artwork::{SongArtworkDao, SongArtworkDaoImpl},
    converts::DbDate,
    song::{SongDao, SongDaoImpl, SongEntry},
    tag::{SongTagsDao, SongTagsDaoImpl},
};

mocks! {
    SongListerFilterImpl,
    []
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
    fn insert_song(&self, entry: &SongEntry) -> i32 {
        self.song_dao.insert(&self.tx, entry).unwrap()
    }
    fn insert_tag(&self, song_id: i32, tag_id: i32) {
        self.song_tags_dao
            .insert(&self.tx, song_id, tag_id)
            .unwrap()
    }
}

static DUMMY_SONG_PATH: Lazy<LibSongPath> = Lazy::new(|| LibSongPath::new(""));

fn dummy_song() -> SongEntry<'static> {
    SongEntry {
        duration: 0,
        path: (&*DUMMY_SONG_PATH).into(),
        folder_id: FolderIdMayRoot::Root.into(),
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
        entry_date: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap().into(),
    }
}

#[sqlx::test()]
fn test_group(db_pool: PgPool) {
    fn insert_song(
        test_db: &TestDb,
        artist: &str,
        tags: &[i32],
        rating: u8,
        release_date: Option<NaiveDate>,
    ) -> i32 {
        let mut song = dummy_song();
        song.artist = artist;
        song.rating = rating;
        song.release_date = release_date.map(DbDate::from);

        let song_id = test_db.insert_song(&song);
        for tag_id in tags {
            test_db.insert_tag(song_id, *tag_id);
        }

        song_id
    }

    let test_db = TestDb::new(&db_pool).await;

    let hit_1 = insert_song(&test_db, "taro", &[45, 58], 3, None);
    insert_song(
        &test_db,
        "jiro",
        &[45, 58],
        4,
        Some(NaiveDate::from_ymd_opt(2021, 9, 25).unwrap()),
    );
    let hit_2 = insert_song(
        &test_db,
        "taro",
        &[],
        5,
        Some(NaiveDate::from_ymd_opt(1999, 9, 9).unwrap()),
    );
    insert_song(&test_db, "taro", &[8, 9, 10], 0, None);
    insert_song(
        &test_db,
        "3bro",
        &[],
        2,
        Some(NaiveDate::from_ymd_opt(1999, 9, 9).unwrap()),
    );
    let hit_3 = insert_song(
        &test_db,
        "taro",
        &[999],
        0,
        Some(NaiveDate::from_ymd_opt(2021, 9, 25).unwrap()),
    );
    let hit_4 = insert_song(
        &test_db,
        "taro",
        &[8, 9, 10],
        4,
        Some(NaiveDate::from_ymd_opt(2021, 9, 25).unwrap()),
    );

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

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter).unwrap(),
            &[hit_1, hit_2, hit_3, hit_4],
        )
    });
}

#[sqlx::test()]
fn test_str(db_pool: PgPool) {
    fn insert_song(test_db: &TestDb, artist: &str) -> i32 {
        let mut song = dummy_song();
        song.artist = artist;
        test_db.insert_song(&song)
    }

    fn filter(range: StringFilterRange) -> FilterTarget {
        FilterTarget::Artist { range }
    }

    let test_db = TestDb::new(&db_pool).await;

    let song_1 = insert_song(&test_db, "test");
    let song_2 = insert_song(&test_db, "AAtest");
    let song_3 = insert_song(&test_db, "testAA");
    let song_4 = insert_song(&test_db, "AAtestAA");
    let song_5 = insert_song(&test_db, "testAAtestAAtestAAtest");
    let song_6 = insert_song(&test_db, "teAAst");
    let song_7 = insert_song(&test_db, "AAAAAA");
    let song_8 = insert_song(&test_db, "");
    let song_9 = insert_song(&test_db, "te%st");
    let song_10 = insert_song(&test_db, "AAte%stAA");

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(StringFilterRange::Equal {
                    value: "test".to_owned(),
                }),
            )
            .unwrap(),
            &[song_1],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(StringFilterRange::NotEqual {
                    value: "test".to_owned(),
                }),
            )
            .unwrap(),
            &[
                song_2, song_3, song_4, song_5, song_6, song_7, song_8, song_9, song_10,
            ],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(StringFilterRange::Start {
                    value: "test".to_owned(),
                }),
            )
            .unwrap(),
            &[song_1, song_3, song_5],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(StringFilterRange::End {
                    value: "test".to_owned(),
                }),
            )
            .unwrap(),
            &[song_1, song_2, song_5],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(StringFilterRange::Contain {
                    value: "test".to_owned(),
                }),
            )
            .unwrap(),
            &[song_1, song_2, song_3, song_4, song_5],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(StringFilterRange::NotContain {
                    value: "test".to_owned(),
                }),
            )
            .unwrap(),
            &[song_6, song_7, song_8, song_9, song_10],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(StringFilterRange::Equal {
                    value: "te%st".to_owned(),
                }),
            )
            .unwrap(),
            &[song_9],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(StringFilterRange::Contain {
                    value: "te%st".to_owned(),
                }),
            )
            .unwrap(),
            &[song_9, song_10],
        );
    });
}

#[sqlx::test()]
fn test_int(db_pool: PgPool) {
    fn insert_song(test_db: &TestDb, track_number: Option<i32>) -> i32 {
        let mut song = dummy_song();
        song.track_number = track_number;
        test_db.insert_song(&song)
    }

    fn filter(range: IntFilterRange) -> FilterTarget {
        FilterTarget::TrackNumber { range }
    }

    let test_db = TestDb::new(&db_pool).await;

    let _song_0 = insert_song(&test_db, None);
    let song_1 = insert_song(&test_db, Some(1));
    let song_5 = insert_song(&test_db, Some(5));
    let song_9 = insert_song(&test_db, Some(9));
    let song_10 = insert_song(&test_db, Some(10));
    let song_25 = insert_song(&test_db, Some(25));
    let song_123 = insert_song(&test_db, Some(123));

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(IntFilterRange::Equal { value: 9 }))
                .unwrap(),
            &[song_9],
        );
        //※nullは含めない仕様(WalkBase1がそうなっていたので)
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(IntFilterRange::NotEqual { value: 25 }))
                .unwrap(),
            &[song_1, song_5, song_9, song_10, song_123],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(IntFilterRange::LargeEqual { value: 10 }),
            )
            .unwrap(),
            &[song_10, song_25, song_123],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(IntFilterRange::SmallEqual { value: 5 }),
            )
            .unwrap(),
            &[song_1, song_5],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(IntFilterRange::RangeIn { min: 9, max: 25 }),
            )
            .unwrap(),
            &[song_9, song_10, song_25],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(IntFilterRange::RangeOut { min: 5, max: 10 }),
            )
            .unwrap(),
            &[song_1, song_25, song_123],
        );
        // assert_eq_not_orderd(
        //     &t.list_song_id(&test_db.tx, &filter(IntFilterRange::Equal { value: None }))
        //         .unwrap(),
        //     &[song_0],
        // );
        // assert_eq_not_orderd(
        //     &t.list_song_id(
        //         &test_db.tx,
        //         &filter(IntFilterRange::NotEqual { value: None }),
        //     )
        //     .unwrap(),
        //     &[song_1, song_5, song_9, song_10, song_25, song_123],
        // );
        // assert_eq_not_orderd(
        //     &t.list_song_id(
        //         &test_db.tx,
        //         &filter(IntFilterRange::LargeEqual { value: None }),
        //     )
        //     .unwrap(),
        //     &[],
        // );
        // assert_eq_not_orderd(
        //     &t.list_song_id(
        //         &test_db.tx,
        //         &filter(IntFilterRange::RangeIn { min: None, max: 5 }),
        //     )
        //     .unwrap(),
        //     &[],
        // );
        // assert_eq_not_orderd(
        //     &t.list_song_id(
        //         &test_db.tx,
        //         &filter(IntFilterRange::RangeIn { min: 5, max: None }),
        //     )
        //     .unwrap(),
        //     &[],
        // );
    });
}

#[sqlx::test()]
fn test_tag(db_pool: PgPool) {
    fn insert_song(test_db: &TestDb, tags: &[i32]) -> i32 {
        let song_id = test_db.insert_song(&dummy_song());
        for tag_id in tags {
            test_db.insert_tag(song_id, *tag_id);
        }

        song_id
    }

    fn filter(range: TagsFilterRange) -> FilterTarget {
        FilterTarget::Tags { range }
    }

    let test_db = TestDb::new(&db_pool).await;

    let song_0 = insert_song(&test_db, &[]);
    let song_1 = insert_song(&test_db, &[4]);
    let song_2 = insert_song(&test_db, &[4, 83]);
    let song_3 = insert_song(&test_db, &[8, 83]);

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(TagsFilterRange::Contain { value: 4 }))
                .unwrap(),
            &[song_1, song_2],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(TagsFilterRange::NotContain { value: 4 }),
            )
            .unwrap(),
            &[song_0, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(TagsFilterRange::Contain { value: 5 }))
                .unwrap(),
            &[],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(TagsFilterRange::NotContain { value: 5 }),
            )
            .unwrap(),
            &[song_0, song_1, song_2, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(TagsFilterRange::Contain { value: 83 }))
                .unwrap(),
            &[song_2, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(TagsFilterRange::NotContain { value: 83 }),
            )
            .unwrap(),
            &[song_0, song_1],
        );

        // assert_eq_not_orderd(
        //     &t.list_song_id(
        //         &test_db.tx,
        //         &filter(TagsFilterRange::Contain { value: None }),
        //     )
        //     .unwrap(),
        //     &[],
        // );
        // assert_eq_not_orderd(
        //     &t.list_song_id(
        //         &test_db.tx,
        //         &filter(TagsFilterRange::NotContain { value: None }),
        //     )
        //     .unwrap(),
        //     &[],
        // );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(TagsFilterRange::None))
                .unwrap(),
            &[song_0],
        );
    });
}

#[sqlx::test()]
fn test_bool(db_pool: PgPool) {
    fn insert_song(test_db: &TestDb, suggest_target: bool) -> i32 {
        let mut song = dummy_song();
        song.suggest_target = suggest_target;
        test_db.insert_song(&song)
    }

    fn filter(range: BoolFilterRange) -> FilterTarget {
        FilterTarget::SuggestTarget { range }
    }

    let test_db = TestDb::new(&db_pool).await;

    let song_true = insert_song(&test_db, true);
    let song_false = insert_song(&test_db, false);

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(BoolFilterRange::True))
                .unwrap(),
            &[song_true],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(BoolFilterRange::False))
                .unwrap(),
            &[song_false],
        );
    });
}

#[sqlx::test()]
fn test_artwork(db_pool: PgPool) {
    fn insert_song(test_db: &TestDb, artworks: &[i32]) -> i32 {
        let song_id = test_db.insert_song(&dummy_song());
        for (idx, artwork_id) in artworks.iter().enumerate() {
            test_db
                .song_artwork_dao
                .insert(&test_db.tx, song_id, idx, *artwork_id, 3 + (idx as u8), "")
                .unwrap();
        }

        song_id
    }
    fn filter(range: ArtworkFilterRange) -> FilterTarget {
        FilterTarget::Artwork { range }
    }

    let test_db = TestDb::new(&db_pool).await;

    let song_0 = insert_song(&test_db, &[]);
    let song_1 = insert_song(&test_db, &[7]);
    let song_2 = insert_song(&test_db, &[5, 6]);

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(ArtworkFilterRange::Has))
                .unwrap(),
            &[song_1, song_2],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(ArtworkFilterRange::None))
                .unwrap(),
            &[song_0],
        );
    });
}

#[sqlx::test()]
fn test_date(db_pool: PgPool) {
    fn insert_song(test_db: &TestDb, release_date: Option<NaiveDate>) -> i32 {
        let mut song = dummy_song();
        song.release_date = release_date.map(DbDate::from);
        test_db.insert_song(&song)
    }

    fn filter(range: DateFilterRange) -> FilterTarget {
        FilterTarget::ReleaseDate { range }
    }

    let test_db = TestDb::new(&db_pool).await;

    let song_0 = insert_song(&test_db, None);
    let song_1 = insert_song(
        &test_db,
        Some(NaiveDate::from_ymd_opt(1998, 12, 10).unwrap()),
    );
    let song_2 = insert_song(&test_db, Some(NaiveDate::from_ymd_opt(2012, 4, 5).unwrap()));
    let song_3 = insert_song(
        &test_db,
        Some(NaiveDate::from_ymd_opt(2021, 9, 26).unwrap()),
    );

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(DateFilterRange::Equal {
                    value: NaiveDate::from_ymd_opt(2012, 4, 5).unwrap(),
                }),
            )
            .unwrap(),
            &[song_2],
        );
        //※nullは含めない仕様(WalkBase1がそうなっていたので)
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(DateFilterRange::NotEqual {
                    value: NaiveDate::from_ymd_opt(2012, 4, 5).unwrap(),
                }),
            )
            .unwrap(),
            &[song_1, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(DateFilterRange::Before {
                    value: NaiveDate::from_ymd_opt(2012, 11, 12).unwrap(),
                }),
            )
            .unwrap(),
            &[song_1, song_2],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter(DateFilterRange::After {
                    value: NaiveDate::from_ymd_opt(2012, 4, 5).unwrap(),
                }),
            )
            .unwrap(),
            &[song_2, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(DateFilterRange::None))
                .unwrap(),
            &[song_0],
        );

        // assert_eq_not_orderd(
        //     &t.list_song_id(&test_db.tx, &filter(DateFilterRange::Equal{value: None}))
        //         .unwrap(),
        //     &[song_0],
        // );
        // assert_eq_not_orderd(
        //     &t.list_song_id(&test_db.tx, &filter(DateFilterRange::NotEqual{value: None}))
        //         .unwrap(),
        //     &[song_1, song_2, song_3],
        // );
        // assert_eq_not_orderd(
        //     &t.list_song_id(&test_db.tx, &filter(DateFilterRange::Before{value: None}))
        //         .unwrap(),
        //     &[],
        // );
        // assert_eq_not_orderd(
        //     &t.list_song_id(&test_db.tx, &filter(DateFilterRange::After{value: None}))
        //         .unwrap(),
        //     &[],
        // );
    });
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
