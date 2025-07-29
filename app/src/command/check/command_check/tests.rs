use std::fs;
use std::str::FromStr;

use murack_core_domain::{
    NonEmptyString, sync::MockDbTrackSyncRepository, test_utils::assert_eq_not_orderd,
    track::MockDbTrackRepository,
};

use super::super::{MockResolveDap, MockResolveDataMatch, MockResolveExistance};
use super::*;
use crate::cui::BufferCui;

fn target<'config, 'cui>(
    arg_path: Option<NonEmptyString>,
    ignore_dap_content: bool,
    config: &'config Config,
    cui: &'cui BufferCui,
) -> CommandCheck<
    'config,
    'cui,
    BufferCui,
    MockResolveExistance,
    MockResolveDataMatch,
    MockResolveDap,
    MockDbTrackRepository,
    MockDbTrackSyncRepository,
> {
    CommandCheck {
        args: CommandCheckArgs {
            path: arg_path,
            ignore_dap_content,
        },
        config,
        cui,
        resolve_existance: MockResolveExistance::default(),
        resolve_data_match: MockResolveDataMatch::default(),
        resolve_dap: MockResolveDap::default(),
        db_track_repository: MockDbTrackRepository::default(),
        db_track_sync_repository: MockDbTrackSyncRepository::default(),
    }
}

fn checkpoint_all(
    target: &mut CommandCheck<
        BufferCui,
        MockResolveExistance,
        MockResolveDataMatch,
        MockResolveDap,
        MockDbTrackRepository,
        MockDbTrackSyncRepository,
    >,
) {
    target.resolve_existance.checkpoint();
    target.resolve_data_match.checkpoint();
    target.resolve_dap.checkpoint();
    target.db_track_sync_repository.inner.checkpoint();
}

#[sqlx::test]
fn test_listup_track_path_green(db_pool: PgPool) -> anyhow::Result<()> {
    fn search_path() -> NonEmptyString {
        NonEmptyString::from_str("test/hoge").unwrap()
    }

    // temp ディレクトリを作成
    let temp_dir = tempfile::tempdir()?;

    // PC ライブラリ側に空ファイルを用意
    let pc_lib = temp_dir.path().join("pc_lib");
    fs::create_dir_all(pc_lib.join("test/hoge/child"))?;
    fs::write(pc_lib.join("test/hoge/child/track3.flac"), "")?;
    fs::write(pc_lib.join("test/hoge/child/track4.flac"), "")?;
    fs::write(pc_lib.join("test/hoge/track1.flac"), "")?;
    fs::write(pc_lib.join("test/hoge/track2.flac"), "")?;

    // DAP ライブラリ側に空ファイルを用意
    let dap_lib = temp_dir.path().join("dap_lib");
    fs::create_dir_all(dap_lib.join("test/hoge/child"))?;
    fs::write(dap_lib.join("test/hoge/child/track3.flac"), "")?;
    fs::write(dap_lib.join("test/hoge/child/track4.flac"), "")?;
    fs::write(dap_lib.join("test/hoge/track1.flac"), "")?;
    fs::write(dap_lib.join("test/hoge/track2.flac"), "")?;

    // tempdir のパスを config に書いておく
    let mut config = Config::dummy();
    config.pc_lib = pc_lib;
    config.dap_lib = dap_lib;

    let cui = BufferCui::new();
    let mut target = target(Some(search_path()), false, &config, &cui);

    // DB 側から返すパスリストを指定
    target
        .db_track_repository
        .inner
        .expect_get_path_by_path_str()
        .times(1)
        .returning(|search| {
            assert_eq!(search, &search_path());
            //なんとなく逆順
            Ok(vec![
                LibraryTrackPath::from_str("test/hoge/track2.flac")?,
                LibraryTrackPath::from_str("test/hoge/track1.flac")?,
                LibraryTrackPath::from_str("test/hoge/child/track4.flac")?,
                LibraryTrackPath::from_str("test/hoge/child/track3.flac")?,
            ])
        });

    assert_eq_not_orderd(
        &target.listup_track_path(&db_pool).await?,
        &[
            LibraryTrackPath::from_str("test/hoge/child/track3.flac")?,
            LibraryTrackPath::from_str("test/hoge/child/track4.flac")?,
            LibraryTrackPath::from_str("test/hoge/track1.flac")?,
            LibraryTrackPath::from_str("test/hoge/track2.flac")?,
        ],
    );

    checkpoint_all(&mut target);
    Ok(())
}

#[sqlx::test]
fn test_listup_track_path_conflict(db_pool: PgPool) -> anyhow::Result<()> {
    // temp ディレクトリを作成
    let temp_dir = tempfile::tempdir()?;

    // PC ライブラリ側に空ファイルを用意
    let pc_lib = temp_dir.path().join("pc_lib");
    fs::create_dir_all(pc_lib.join("test/hoge/child"))?;
    fs::write(pc_lib.join("test/hoge/child/track1.flac"), "")?;
    fs::write(pc_lib.join("test/hoge/child/pc1.flac"), "")?;
    fs::write(pc_lib.join("test/hoge/track2.flac"), "")?;
    fs::write(pc_lib.join("test/hoge/pc2.flac"), "")?;

    // DAP ライブラリ側に空ファイルを用意
    let dap_lib = temp_dir.path().join("dap_lib");
    fs::create_dir_all(dap_lib.join("test/hoge/child"))?;
    fs::write(dap_lib.join("test/hoge/child/track1.flac"), "")?;
    fs::write(dap_lib.join("test/hoge/child/dap1.flac"), "")?;
    fs::write(dap_lib.join("test/hoge/track2.flac"), "")?;

    // tempdir のパスを config に書いておく
    let mut config = Config::dummy();
    config.pc_lib = pc_lib;
    config.dap_lib = dap_lib;

    let cui = BufferCui::new();
    let arg_path = Some(NonEmptyString::from_str("test/hoge")?);
    let mut target = target(arg_path, false, &config, &cui);

    // DB 側から返すパスリストを指定
    target
        .db_track_repository
        .inner
        .expect_get_path_by_path_str()
        .returning(|_| {
            Ok(vec![
                LibraryTrackPath::from_str("test/hoge/child/track1.flac")?,
                LibraryTrackPath::from_str("test/hoge/track2.flac")?,
                LibraryTrackPath::from_str("test/hoge/db1.flac")?,
            ])
        });

    assert_eq!(
        target.listup_track_path(&db_pool).await?,
        vec![
            LibraryTrackPath::from_str("test/hoge/child/dap1.flac")?,
            LibraryTrackPath::from_str("test/hoge/child/pc1.flac")?,
            LibraryTrackPath::from_str("test/hoge/child/track1.flac")?,
            LibraryTrackPath::from_str("test/hoge/db1.flac")?,
            LibraryTrackPath::from_str("test/hoge/pc2.flac")?,
            LibraryTrackPath::from_str("test/hoge/track2.flac")?,
        ]
    );

    checkpoint_all(&mut target);
    Ok(())
}
