//! SongListerFilterのテスト

use super::*;
use crate::{
    artwork::{SongArtworkDao, SongArtworkDaoImpl},
    converts::DbDate,
    initialize,
    song::{SongDao, SongDaoImpl, SongEntry},
    tag::{SongTagsDao, SongTagsDaoImpl},
};
use chrono::NaiveDate;
use domain::{
    db_wrapper::{ConnectionFactory, ConnectionWrapper},
    folder::FolderIdMayRoot,
    mocks,
    path::LibSongPath,
    test_utils::assert_eq_not_orderd,
};
use once_cell::sync::Lazy;
use paste::paste;
use test_case::test_case;

mocks! {
    SongListerFilterImpl,
    []
}

struct TestDb<'c> {
    tx: TransactionWrapper<'c>,
    song_dao: SongDaoImpl,
    song_tags_dao: SongTagsDaoImpl,
    song_artwork_dao: SongArtworkDaoImpl,
}
impl<'c> TestDb<'c> {
    fn new(db: &'c mut ConnectionWrapper) -> Self {
        let tx = db.transaction().unwrap();

        initialize::song(&tx).unwrap();
        initialize::song_tags(&tx).unwrap();
        initialize::song_artwork(&tx).unwrap();

        Self {
            tx,
            song_dao: SongDaoImpl {},
            song_tags_dao: SongTagsDaoImpl {},
            song_artwork_dao: SongArtworkDaoImpl {},
        }
    }
}
impl TestDb<'_> {
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

#[test]
fn test_group() {
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

    let mut db = ConnectionFactory::Memory.open().unwrap();
    let test_db = TestDb::new(&mut db);

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

    let filter = Filter {
        rowid: 0,
        target: FilterTarget::FilterGroup,
        str_value: String::default(),
        str_value2: String::default(),
        range: FilterValueRange::GroupAnd,
        children: vec![
            Filter {
                rowid: 1,
                target: FilterTarget::Artist,
                str_value: "taro".to_owned(),
                str_value2: String::default(),
                range: FilterValueRange::StrContain,
                children: vec![],
            },
            Filter {
                rowid: 2,
                target: FilterTarget::FilterGroup,
                str_value: String::default(),
                str_value2: String::default(),
                range: FilterValueRange::GroupOr,
                children: vec![
                    Filter {
                        rowid: 3,
                        target: FilterTarget::Tags,
                        str_value: "45".to_owned(),
                        str_value2: String::default(),
                        range: FilterValueRange::TagContain,
                        children: vec![],
                    },
                    Filter {
                        rowid: 9,
                        target: FilterTarget::Rating,
                        str_value: "4".to_owned(),
                        str_value2: String::default(),
                        range: FilterValueRange::IntLargeEqual,
                        children: vec![],
                    },
                    Filter {
                        rowid: 21,
                        target: FilterTarget::ReleaseDate,
                        str_value: "2021-09-25".to_owned(),
                        str_value2: String::default(),
                        range: FilterValueRange::DateEqual,
                        children: vec![],
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

#[test]
fn test_str() {
    fn insert_song(test_db: &TestDb, artist: &str) -> i32 {
        let mut song = dummy_song();
        song.artist = artist;
        test_db.insert_song(&song)
    }

    fn filter(value: &str, range: FilterValueRange) -> Filter {
        Filter {
            str_value: value.to_owned(),
            range,

            rowid: 1,
            target: FilterTarget::Artist,
            str_value2: String::default(),
            children: vec![],
        }
    }

    let mut db = ConnectionFactory::Memory.open().unwrap();
    let test_db = TestDb::new(&mut db);

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
            &t.list_song_id(&test_db.tx, &filter("test", FilterValueRange::StrEqual))
                .unwrap(),
            &[song_1],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("test", FilterValueRange::StrNotEqual))
                .unwrap(),
            &[
                song_2, song_3, song_4, song_5, song_6, song_7, song_8, song_9, song_10,
            ],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("test", FilterValueRange::StrStart))
                .unwrap(),
            &[song_1, song_3, song_5],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("test", FilterValueRange::StrEnd))
                .unwrap(),
            &[song_1, song_2, song_5],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("test", FilterValueRange::StrContain))
                .unwrap(),
            &[song_1, song_2, song_3, song_4, song_5],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter("test", FilterValueRange::StrNotContain),
            )
            .unwrap(),
            &[song_6, song_7, song_8, song_9, song_10],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("te%st", FilterValueRange::StrEqual))
                .unwrap(),
            &[song_9],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("te%st", FilterValueRange::StrContain))
                .unwrap(),
            &[song_9, song_10],
        );
    });
}

#[test]
fn test_int() {
    fn insert_song(test_db: &TestDb, track_number: Option<i32>) -> i32 {
        let mut song = dummy_song();
        song.track_number = track_number;
        test_db.insert_song(&song)
    }

    fn filter(value1: &str, value2: &str, range: FilterValueRange) -> Filter {
        Filter {
            str_value: value1.to_owned(),
            str_value2: value2.to_owned(),
            range,

            rowid: 1,
            target: FilterTarget::TrackNumber,
            children: vec![],
        }
    }

    let mut db = ConnectionFactory::Memory.open().unwrap();
    let test_db = TestDb::new(&mut db);

    let song_0 = insert_song(&test_db, None);
    let song_1 = insert_song(&test_db, Some(1));
    let song_5 = insert_song(&test_db, Some(5));
    let song_9 = insert_song(&test_db, Some(9));
    let song_10 = insert_song(&test_db, Some(10));
    let song_25 = insert_song(&test_db, Some(25));
    let song_123 = insert_song(&test_db, Some(123));

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("9", "", FilterValueRange::IntEqual))
                .unwrap(),
            &[song_9],
        );
        //※nullは含めない仕様(WalkBase1がそうなっていたので)
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter("25", "", FilterValueRange::IntNotEqual),
            )
            .unwrap(),
            &[song_1, song_5, song_9, song_10, song_123],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter("10", "", FilterValueRange::IntLargeEqual),
            )
            .unwrap(),
            &[song_10, song_25, song_123],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter("5", "", FilterValueRange::IntSmallEqual),
            )
            .unwrap(),
            &[song_1, song_5],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter("9", "25", FilterValueRange::IntRangeIn),
            )
            .unwrap(),
            &[song_9, song_10, song_25],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter("5", "10", FilterValueRange::IntRangeOut),
            )
            .unwrap(),
            &[song_1, song_25, song_123],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", "", FilterValueRange::IntEqual))
                .unwrap(),
            &[song_0],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", "", FilterValueRange::IntNotEqual))
                .unwrap(),
            &[song_1, song_5, song_9, song_10, song_25, song_123],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter("", "", FilterValueRange::IntLargeEqual),
            )
            .unwrap(),
            &[],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", "5", FilterValueRange::IntRangeIn))
                .unwrap(),
            &[],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("5", "", FilterValueRange::IntRangeIn))
                .unwrap(),
            &[],
        );
    });
}

#[test]
fn test_tag() {
    fn insert_song(test_db: &TestDb, tags: &[i32]) -> i32 {
        let song_id = test_db.insert_song(&dummy_song());
        for tag_id in tags {
            test_db.insert_tag(song_id, *tag_id);
        }

        song_id
    }

    fn filter(value: &str, range: FilterValueRange) -> Filter {
        Filter {
            str_value: value.to_owned(),
            range,

            rowid: 1,
            target: FilterTarget::Tags,
            str_value2: String::default(),
            children: vec![],
        }
    }

    let mut db = ConnectionFactory::Memory.open().unwrap();
    let test_db = TestDb::new(&mut db);

    let song_0 = insert_song(&test_db, &[]);
    let song_1 = insert_song(&test_db, &[4]);
    let song_2 = insert_song(&test_db, &[4, 83]);
    let song_3 = insert_song(&test_db, &[8, 83]);

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("4", FilterValueRange::TagContain))
                .unwrap(),
            &[song_1, song_2],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("4", FilterValueRange::TagNotContain))
                .unwrap(),
            &[song_0, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("5", FilterValueRange::TagContain))
                .unwrap(),
            &[],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("5", FilterValueRange::TagNotContain))
                .unwrap(),
            &[song_0, song_1, song_2, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("83", FilterValueRange::TagContain))
                .unwrap(),
            &[song_2, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("83", FilterValueRange::TagNotContain))
                .unwrap(),
            &[song_0, song_1],
        );

        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", FilterValueRange::TagContain))
                .unwrap(),
            &[],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", FilterValueRange::TagNotContain))
                .unwrap(),
            &[],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", FilterValueRange::TagNone))
                .unwrap(),
            &[song_0],
        );
    });
}

#[test]
fn test_bool() {
    fn insert_song(test_db: &TestDb, suggest_target: bool) -> i32 {
        let mut song = dummy_song();
        song.suggest_target = suggest_target;
        test_db.insert_song(&song)
    }

    fn filter(range: FilterValueRange) -> Filter {
        Filter {
            range,

            rowid: 1,
            target: FilterTarget::SuggestTarget,
            str_value: String::default(),
            str_value2: String::default(),
            children: vec![],
        }
    }

    let mut db = ConnectionFactory::Memory.open().unwrap();
    let test_db = TestDb::new(&mut db);

    let song_true = insert_song(&test_db, true);
    let song_false = insert_song(&test_db, false);

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(FilterValueRange::BoolTrue))
                .unwrap(),
            &[song_true],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(FilterValueRange::BoolFalse))
                .unwrap(),
            &[song_false],
        );
    });
}

#[test]
fn test_artwork() {
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
    fn filter(range: FilterValueRange) -> Filter {
        Filter {
            range,

            rowid: 1,
            target: FilterTarget::Artwork,
            str_value: String::default(),
            str_value2: String::default(),
            children: vec![],
        }
    }

    let mut db = ConnectionFactory::Memory.open().unwrap();
    let test_db = TestDb::new(&mut db);

    let song_0 = insert_song(&test_db, &[]);
    let song_1 = insert_song(&test_db, &[7]);
    let song_2 = insert_song(&test_db, &[5, 6]);

    let mut mocks = Mocks::new();
    mocks.run_target(|t| {
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(FilterValueRange::ArtworkHas))
                .unwrap(),
            &[song_1, song_2],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter(FilterValueRange::ArtworkNone))
                .unwrap(),
            &[song_0],
        );
    });
}

#[test]
fn test_date() {
    fn insert_song(test_db: &TestDb, release_date: Option<NaiveDate>) -> i32 {
        let mut song = dummy_song();
        song.release_date = release_date.map(DbDate::from);
        test_db.insert_song(&song)
    }

    fn filter(value: &str, range: FilterValueRange) -> Filter {
        Filter {
            str_value: value.to_owned(),
            range,

            rowid: 1,
            target: FilterTarget::ReleaseDate,
            str_value2: String::default(),
            children: vec![],
        }
    }

    let mut db = ConnectionFactory::Memory.open().unwrap();
    let test_db = TestDb::new(&mut db);

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
                &filter("2012-04-05", FilterValueRange::DateEqual),
            )
            .unwrap(),
            &[song_2],
        );
        //※nullは含めない仕様(WalkBase1がそうなっていたので)
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter("2012-04-05", FilterValueRange::DateNotEqual),
            )
            .unwrap(),
            &[song_1, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter("2012-11-12", FilterValueRange::DateBefore),
            )
            .unwrap(),
            &[song_1, song_2],
        );
        assert_eq_not_orderd(
            &t.list_song_id(
                &test_db.tx,
                &filter("2012-04-05", FilterValueRange::DateAfter),
            )
            .unwrap(),
            &[song_2, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", FilterValueRange::DateNone))
                .unwrap(),
            &[song_0],
        );

        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", FilterValueRange::DateEqual))
                .unwrap(),
            &[song_0],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", FilterValueRange::DateNotEqual))
                .unwrap(),
            &[song_1, song_2, song_3],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", FilterValueRange::DateBefore))
                .unwrap(),
            &[],
        );
        assert_eq_not_orderd(
            &t.list_song_id(&test_db.tx, &filter("", FilterValueRange::DateAfter))
                .unwrap(),
            &[],
        );
    });
}

#[test_case("15", "28", 15, 28 ; "normal")]
#[test_case("28", "15", 15, 28 ; "inversed")]
#[test_case("6", "113", 6, 113 ; "digit_dif")]
#[test_case("113", "6", 6, 113 ; "digit_dif_inversed")]
fn test_get_ordered_int(value1: &str, value2: &str, expect_1: i32, expect_2: i32) {
    let filter = Filter {
        str_value: value1.to_owned(),
        str_value2: value2.to_owned(),

        rowid: 1,
        target: FilterTarget::DiscNumber,
        range: FilterValueRange::IntRangeIn,
        children: vec![],
    };
    let result = get_ordered_int(&filter).unwrap();
    assert_eq!(result.0, expect_1);
    assert_eq!(result.1, expect_2);
}
