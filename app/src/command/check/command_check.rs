//! checkコマンド
//!
//! PC・DAP・DBの齟齬を確認する

use super::{
    Args, ResolveDap, ResolveDapImpl, ResolveDataMatch, ResolveDataMatchImpl, ResolveExistance,
    ResolveExistanceImpl, ResolveFileExistanceResult,
};
use crate::{AppComponents, Config, cui::Cui};
use anyhow::Result;
use domain::{
    FileLibraryRepository,
    check::{CheckIssueSummary, CheckUsecase},
    db_wrapper::{ConnectionFactory, ConnectionWrapper},
    path::LibSongPath,
    song::DbSongRepository,
};
use std::collections::BTreeSet;
use std::rc::Rc;

pub struct CommandCheck {
    args: Args,

    resolve_existance: Rc<dyn ResolveExistance>,
    resolve_data_match: Rc<dyn ResolveDataMatch>,
    resolve_dap: Rc<dyn ResolveDap>,

    config: Rc<Config>,
    cui: Rc<dyn Cui>,
    connection_factory: Rc<ConnectionFactory>,
    file_library_repository: Rc<dyn FileLibraryRepository>,
    check_usecase: Rc<dyn CheckUsecase>,
    db_song_repository: Rc<dyn DbSongRepository>,
}

impl CommandCheck {
    pub fn new(command_line: &[String], app_components: &impl AppComponents) -> Result<Self> {
        Ok(Self {
            args: Args::parse(command_line)?,
            resolve_existance: Rc::new(ResolveExistanceImpl::new(app_components)),
            resolve_data_match: Rc::new(ResolveDataMatchImpl::new(app_components)),
            resolve_dap: Rc::new(ResolveDapImpl::new(app_components)),
            config: app_components.config().clone(),
            cui: app_components.cui().clone(),
            connection_factory: app_components.connection_factory().clone(),
            file_library_repository: app_components.file_library_repository().clone(),
            check_usecase: app_components.check_usecase().clone(),
            db_song_repository: app_components.db_song_repository().clone(),
        })
    }

    /// このコマンドを実行
    pub fn run(&self) -> Result<()> {
        let mut db = self.connection_factory.open()?;

        let path_list = self.listup_song_path(&mut db)?;
        let conflict_list = self.summary_check(&mut db, path_list)?;

        if !self.summary_result_cui(&conflict_list)? {
            return Ok(());
        }

        let terminated = self.resolve_all_songs(&mut db, &conflict_list)?;

        if !terminated {
            let cui = &self.cui;

            cui_outln!(cui, "====================");
            cui_outln!(cui, "全ての問題の解決処理が終了しました。");
        }

        Ok(())
    }

    /// 全ての対象曲をリストアップ
    /// # Returns
    /// 全対象曲のパス
    fn listup_song_path(&self, db: &mut ConnectionWrapper) -> Result<Vec<LibSongPath>> {
        let cui = &self.cui;

        //マージ用set
        let mut set = BTreeSet::<LibSongPath>::new();

        //PCからリストアップ
        cui_outln!(cui, "PCの検索中...");
        for path in self
            .file_library_repository
            .search_by_lib_path(&self.config.pc_lib, &self.args.path)?
        {
            set.insert(path);
        }

        //DAPからリストアップ
        cui_outln!(cui, "DAPの検索中...");
        for path in self
            .file_library_repository
            .search_by_lib_path(&self.config.dap_lib, &self.args.path)?
        {
            set.insert(path);
        }

        //DBからリストアップ
        cui_outln!(cui, "DBの検索中...");

        db.run_in_transaction(|tx| {
            for path in self
                .db_song_repository
                .get_path_by_path_str(tx, &self.args.path)?
            {
                set.insert(path);
            }

            Ok(())
        })?;

        Ok(set.into_iter().collect())
    }

    /// 対象曲全体の簡易チェック
    /// # Arguments
    /// - path_list: チェック対象の全曲のパス
    /// # Returns
    /// 問題があった曲のパスリスト
    fn summary_check(
        &self,
        db: &mut ConnectionWrapper,
        path_list: Vec<LibSongPath>,
    ) -> Result<Vec<LibSongPath>> {
        let cui = &self.cui;

        let mut conflict_list = Vec::<(LibSongPath, Vec<CheckIssueSummary>)>::new();

        //全曲に対して整合性チェック
        let all_count = path_list.len();
        for (current_index, path) in path_list.into_iter().enumerate() {
            if current_index % 100 == 0 {
                cui_outln!(cui, "チェック中...({}/{})", current_index, all_count);
            }

            let issues = self.check_usecase.listup_issue_summary(
                db,
                &self.config.pc_lib,
                &self.config.dap_lib,
                &path,
                self.args.ignore_dap_content,
            )?;

            if !issues.is_empty() {
                conflict_list.push((path, issues));
            }
        }

        cui_outln!(cui, "チェック中...({}/{})", all_count, all_count);

        //全ファイルのチェックが終わった後で、簡易結果出力
        for (path, issues) in &conflict_list {
            cui_outln!(cui, "# {}", path);
            for issue in issues {
                cui_outln!(cui, "---- {}", issue);
            }
        }
        cui_outln!(cui);

        Ok(conflict_list.into_iter().map(|(path, _)| path).collect())
    }

    /// 簡易チェックの結果確認CUI処理
    /// # Returns
    /// 次の解決処理に進むならtrue
    fn summary_result_cui(&self, conflict_list: &[LibSongPath]) -> Result<bool> {
        let cui = &self.cui;

        //齟齬がなければ終了
        if conflict_list.is_empty() {
            cui_outln!(cui, "問題はありませんでした。");
            return Ok(false);
        }

        cui_outln!(
            cui,
            "{}個のファイルで問題を検出しました。",
            conflict_list.len()
        );

        //継続確認
        let result = cui.input_case(&['y', 'n'], "解決処理を行いますか? (y/n)->")?;
        cui_outln!(cui);

        Ok(result == 'y')
    }

    /// 問題があった全ての曲の解決処理
    /// # Returns
    /// 強制終了されたらtrue
    fn resolve_all_songs(
        &self,
        db: &mut ConnectionWrapper,
        conflict_list: &[LibSongPath],
    ) -> Result<bool> {
        let all_count = conflict_list.len();
        for (current_index, song_path) in conflict_list.iter().enumerate() {
            {
                let cui = &self.cui;

                cui_outln!(cui, "====================");
                cui_outln!(cui, "{}", song_path);
                cui_outln!(cui, "({}/{})", current_index + 1, all_count);
                cui_outln!(cui);
            }

            if !self.resolve_song(db, song_path)? {
                return Ok(true);
            }

            let cui = &self.cui;
            cui_outln!(cui);
        }

        Ok(false)
    }

    /// 1曲についての問題の全解決処理の実行
    ///
    /// # Arguments
    /// - song_path: 作業対象の曲のパス
    ///
    /// # Returns
    /// 次の曲の解決処理へ継続するか
    fn resolve_song(&self, db: &mut ConnectionWrapper, song_path: &LibSongPath) -> Result<bool> {
        //存在チェック・解決処理
        match self.resolve_existance.resolve(db, song_path)? {
            ResolveFileExistanceResult::Resolved => {}
            ResolveFileExistanceResult::Deleted => return Ok(true),
            ResolveFileExistanceResult::UnResolved => return Ok(true),
            ResolveFileExistanceResult::Terminated => return Ok(false),
        }

        //データ同一性の解決処理
        if !self.resolve_data_match.resolve(db, song_path)? {
            return Ok(false);
        }

        //PC・DAP間の齟齬の解決処理
        if !self.resolve_dap.resolve_pc_dap_conflict(song_path)? {
            return Ok(false);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{MockResolveDap, MockResolveDataMatch, MockResolveExistance};
    use super::*;
    use crate::cui::BufferCui;
    use domain::{
        MockFileLibraryRepository, check::MockCheckUsecase, mocks, path::LibPathStr,
        song::MockDbSongRepository,
    };
    use paste::paste;
    use std::path::PathBuf;

    mocks! {
        CommandCheck,
        [ResolveExistance, ResolveDataMatch, ResolveDap, FileLibraryRepository, CheckUsecase, DbSongRepository],
        [args: Args, config: Rc<Config>, cui: Rc<BufferCui>, connection_factory: Rc<ConnectionFactory>]
    }

    fn new_mocks(arg_path: LibPathStr, ignore_dap_content: bool) -> Mocks {
        Mocks::new(
            Args {
                path: arg_path,
                ignore_dap_content,
            },
            Rc::new(Config::dummy()),
            Rc::new(BufferCui::new()),
            Rc::new(ConnectionFactory::Dummy),
        )
    }

    fn pc_lib() -> PathBuf {
        "pc_lib".into()
    }
    fn dap_lib() -> PathBuf {
        "dap_lib".into()
    }

    #[test]
    fn test_listup_song_path_green() {
        fn search_path() -> LibPathStr {
            "test/hoge".to_owned().into()
        }
        fn song_paths() -> Vec<LibSongPath> {
            vec![
                LibSongPath::new("test/hoge/child/song3.flac"),
                LibSongPath::new("test/hoge/child/song4.flac"),
                LibSongPath::new("test/hoge/song1.flac"),
                LibSongPath::new("test/hoge/song2.flac"),
            ]
        }

        let mut mocks = new_mocks(search_path(), false);

        mocks.file_library_repository(|m| {
            m.expect_search_by_lib_path()
                .withf(|lib, _| lib == pc_lib())
                .times(1)
                .returning(|_, search| {
                    assert_eq!(search, &search_path());
                    Ok(song_paths())
                });
            m.expect_search_by_lib_path()
                .withf(|lib, _| lib == dap_lib())
                .times(1)
                .returning(|_, search| {
                    assert_eq!(search, &search_path());
                    Ok(song_paths())
                });
        });
        mocks.db_song_repository(|m| {
            m.expect_get_path_by_path_str()
                .times(1)
                .returning(|_, search| {
                    assert_eq!(search, &search_path());
                    //なんとなく逆順
                    Ok(song_paths().into_iter().rev().collect())
                });
        });

        let mut db = mocks.connection_factory.open().unwrap();

        mocks.run_target(|target| {
            assert_eq!(target.listup_song_path(&mut db).unwrap(), song_paths())
        });
    }

    #[test]
    fn test_listup_song_path_conflict() {
        let mut mocks = new_mocks("test/hoge".to_owned().into(), false);

        mocks.file_library_repository(|m| {
            m.expect_search_by_lib_path()
                .withf(|lib, _| lib == pc_lib())
                .returning(|_, _| {
                    Ok(vec![
                        LibSongPath::new("test/hoge/child/song1.flac"),
                        LibSongPath::new("test/hoge/child/pc1.flac"),
                        LibSongPath::new("test/hoge/song2.flac"),
                        LibSongPath::new("test/hoge/pc2.flac"),
                    ])
                });
            m.expect_search_by_lib_path()
                .withf(|lib, _| lib == dap_lib())
                .returning(|_, _| {
                    Ok(vec![
                        LibSongPath::new("test/hoge/child/song1.flac"),
                        LibSongPath::new("test/hoge/child/dap1.flac"),
                        LibSongPath::new("test/hoge/song2.flac"),
                    ])
                });
        });
        mocks.db_song_repository(|m| {
            m.expect_get_path_by_path_str().returning(|_, _| {
                Ok(vec![
                    LibSongPath::new("test/hoge/child/song1.flac"),
                    LibSongPath::new("test/hoge/song2.flac"),
                    LibSongPath::new("test/hoge/db1.flac"),
                ])
            });
        });

        let mut db = mocks.connection_factory.open().unwrap();

        mocks.run_target(|target| {
            assert_eq!(
                target.listup_song_path(&mut db).unwrap(),
                vec![
                    LibSongPath::new("test/hoge/child/dap1.flac"),
                    LibSongPath::new("test/hoge/child/pc1.flac"),
                    LibSongPath::new("test/hoge/child/song1.flac"),
                    LibSongPath::new("test/hoge/db1.flac"),
                    LibSongPath::new("test/hoge/pc2.flac"),
                    LibSongPath::new("test/hoge/song2.flac"),
                ]
            )
        });
    }
}
