//! checkコマンド
//!
//! PC・DAP・DBの齟齬を確認する

use std::collections::BTreeSet;

use anyhow::Result;
use murack_core_domain::{
    FileLibraryRepository,
    check::{CheckIssueSummary, CheckUsecase},
    path::LibTrackPath,
    track::DbTrackRepository,
};
use sqlx::PgPool;

use super::{
    CommandCheckArgs, ResolveDap, ResolveDataMatch, ResolveExistance, ResolveFileExistanceResult,
};
use crate::{Config, cui::Cui};

pub struct CommandCheck<'config, 'cui, CUI, REX, RDM, RDP, FR, CS, SR>
where
    CUI: Cui + Send + Sync,
    REX: ResolveExistance,
    RDM: ResolveDataMatch,
    RDP: ResolveDap,
    FR: FileLibraryRepository,
    CS: CheckUsecase,
    SR: DbTrackRepository,
{
    args: CommandCheckArgs,

    resolve_existance: REX,
    resolve_data_match: RDM,
    resolve_dap: RDP,

    config: &'config Config,
    cui: &'cui CUI,
    file_library_repository: FR,
    check_usecase: CS,
    db_track_repository: SR,
}

impl<'config, 'cui, CUI, REX, RDM, RDP, FR, CS, SR>
    CommandCheck<'config, 'cui, CUI, REX, RDM, RDP, FR, CS, SR>
where
    CUI: Cui + Send + Sync,
    REX: ResolveExistance,
    RDM: ResolveDataMatch,
    RDP: ResolveDap,
    FR: FileLibraryRepository,
    CS: CheckUsecase,
    SR: DbTrackRepository,
{
    #[allow(clippy::too_many_arguments)] // todo
    pub fn new(
        args: CommandCheckArgs,
        config: &'config Config,

        resolve_existance: REX,
        resolve_data_match: RDM,
        resolve_dap: RDP,
        cui: &'cui CUI,
        file_library_repository: FR,
        check_usecase: CS,
        db_track_repository: SR,
    ) -> Self {
        Self {
            args,
            resolve_existance,
            resolve_data_match,
            resolve_dap,
            config,
            cui,
            file_library_repository,
            check_usecase,
            db_track_repository,
        }
    }

    /// このコマンドを実行
    pub async fn run(&self, db_pool: &PgPool) -> Result<()> {
        let path_list = self.listup_track_path(db_pool).await?;
        let conflict_list = self.summary_check(db_pool, path_list).await?;

        if !self.summary_result_cui(&conflict_list)? {
            return Ok(());
        }

        let terminated = self.resolve_all_tracks(db_pool, &conflict_list).await?;

        if !terminated {
            let cui = &self.cui;

            cui_outln!(cui, "====================")?;
            cui_outln!(cui, "全ての問題の解決処理が終了しました。")?;
        }

        Ok(())
    }

    /// 全ての対象曲をリストアップ
    /// # Returns
    /// 全対象曲のパス
    async fn listup_track_path(&self, db_pool: &PgPool) -> Result<Vec<LibTrackPath>> {
        let cui = &self.cui;

        //マージ用set
        let mut set = BTreeSet::<LibTrackPath>::new();

        //PCからリストアップ
        cui_outln!(cui, "PCの検索中...")?;
        for path in self
            .file_library_repository
            .search_by_lib_path(&self.config.pc_lib, &self.args.path)?
        {
            set.insert(path);
        }

        //DAPからリストアップ
        cui_outln!(cui, "DAPの検索中...")?;
        for path in self
            .file_library_repository
            .search_by_lib_path(&self.config.dap_lib, &self.args.path)?
        {
            set.insert(path);
        }

        //DBからリストアップ
        cui_outln!(cui, "DBの検索中...")?;

        let mut tx = db_pool.begin().await?;

        for path in self
            .db_track_repository
            .get_path_by_path_str(&mut tx, &self.args.path)
            .await?
        {
            set.insert(path);
        }

        Ok(set.into_iter().collect())
    }

    /// 対象曲全体の簡易チェック
    /// # Arguments
    /// - path_list: チェック対象の全曲のパス
    /// # Returns
    /// 問題があった曲のパスリスト
    async fn summary_check(
        &self,
        db_pool: &PgPool,
        path_list: Vec<LibTrackPath>,
    ) -> Result<Vec<LibTrackPath>> {
        let cui = &self.cui;

        let mut conflict_list = Vec::<(LibTrackPath, Vec<CheckIssueSummary>)>::new();

        //全曲に対して整合性チェック
        let all_count = path_list.len();
        for (current_index, path) in path_list.into_iter().enumerate() {
            if current_index % 100 == 0 {
                cui_outln!(cui, "チェック中...({}/{})", current_index, all_count)?;
            }

            let issues = self
                .check_usecase
                .listup_issue_summary(
                    db_pool,
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    &path,
                    self.args.ignore_dap_content,
                )
                .await?;

            if !issues.is_empty() {
                conflict_list.push((path, issues));
            }
        }

        cui_outln!(cui, "チェック中...({}/{})", all_count, all_count)?;

        //全ファイルのチェックが終わった後で、簡易結果出力
        for (path, issues) in &conflict_list {
            cui_outln!(cui, "# {}", path)?;
            for issue in issues {
                cui_outln!(cui, "---- {}", issue)?;
            }
        }
        cui_outln!(cui)?;

        Ok(conflict_list.into_iter().map(|(path, _)| path).collect())
    }

    /// 簡易チェックの結果確認CUI処理
    /// # Returns
    /// 次の解決処理に進むならtrue
    fn summary_result_cui(&self, conflict_list: &[LibTrackPath]) -> Result<bool> {
        let cui = &self.cui;

        //齟齬がなければ終了
        if conflict_list.is_empty() {
            cui_outln!(cui, "問題はありませんでした。")?;
            return Ok(false);
        }

        cui_outln!(
            cui,
            "{}個のファイルで問題を検出しました。",
            conflict_list.len()
        )?;

        //継続確認
        let result = cui.input_case(&['y', 'n'], "解決処理を行いますか? (y/n)->")?;
        cui_outln!(cui)?;

        Ok(result == 'y')
    }

    /// 問題があった全ての曲の解決処理
    /// # Returns
    /// 強制終了されたらtrue
    async fn resolve_all_tracks(
        &self,
        db_pool: &PgPool,
        conflict_list: &[LibTrackPath],
    ) -> Result<bool> {
        let all_count = conflict_list.len();
        for (current_index, track_path) in conflict_list.iter().enumerate() {
            {
                let cui = &self.cui;

                cui_outln!(cui, "====================")?;
                cui_outln!(cui, "{}", track_path)?;
                cui_outln!(cui, "({}/{})", current_index + 1, all_count)?;
                cui_outln!(cui)?;
            }

            if !self.resolve_track(db_pool, track_path).await? {
                return Ok(true);
            }

            let cui = &self.cui;
            cui_outln!(cui)?;
        }

        Ok(false)
    }

    /// 1曲についての問題の全解決処理の実行
    ///
    /// # Arguments
    /// - track_path: 作業対象の曲のパス
    ///
    /// # Returns
    /// 次の曲の解決処理へ継続するか
    async fn resolve_track(&self, db_pool: &PgPool, track_path: &LibTrackPath) -> Result<bool> {
        //存在チェック・解決処理
        match self.resolve_existance.resolve(db_pool, track_path).await? {
            ResolveFileExistanceResult::Resolved => {}
            ResolveFileExistanceResult::Deleted => return Ok(true),
            ResolveFileExistanceResult::UnResolved => return Ok(true),
            ResolveFileExistanceResult::Terminated => return Ok(false),
        }

        //データ同一性の解決処理
        if !self.resolve_data_match.resolve(db_pool, track_path).await? {
            return Ok(false);
        }

        //PC・DAP間の齟齬の解決処理
        if !self.resolve_dap.resolve_pc_dap_conflict(track_path)? {
            return Ok(false);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use murack_core_data_file::FileLibraryRepositoryImpl;
    use murack_core_domain::test_utils::assert_eq_not_orderd;
    use murack_core_domain::{
        check::MockCheckUsecase, path::LibPathStr, track::MockDbTrackRepository,
    };

    use super::super::{MockResolveDap, MockResolveDataMatch, MockResolveExistance};
    use super::*;
    use crate::cui::BufferCui;

    fn target<'config, 'cui>(
        arg_path: LibPathStr,
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
        FileLibraryRepositoryImpl,
        MockCheckUsecase,
        MockDbTrackRepository,
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
            file_library_repository: FileLibraryRepositoryImpl {},
            check_usecase: MockCheckUsecase::default(),
            db_track_repository: MockDbTrackRepository::default(),
        }
    }

    fn checkpoint_all(
        target: &mut CommandCheck<
            BufferCui,
            MockResolveExistance,
            MockResolveDataMatch,
            MockResolveDap,
            FileLibraryRepositoryImpl,
            MockCheckUsecase,
            MockDbTrackRepository,
        >,
    ) {
        target.resolve_existance.checkpoint();
        target.resolve_data_match.checkpoint();
        target.resolve_dap.checkpoint();
        target.check_usecase.checkpoint();
        target.db_track_repository.inner.checkpoint();
    }

    #[sqlx::test]
    fn test_listup_track_path_green(db_pool: PgPool) -> anyhow::Result<()> {
        fn search_path() -> LibPathStr {
            "test/hoge".to_owned().into()
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
        let mut target = target(search_path(), false, &config, &cui);

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
                    LibTrackPath::new("test/hoge/track2.flac"),
                    LibTrackPath::new("test/hoge/track1.flac"),
                    LibTrackPath::new("test/hoge/child/track4.flac"),
                    LibTrackPath::new("test/hoge/child/track3.flac"),
                ])
            });

        assert_eq_not_orderd(
            &target.listup_track_path(&db_pool).await?,
            &[
                LibTrackPath::new("test/hoge/child/track3.flac"),
                LibTrackPath::new("test/hoge/child/track4.flac"),
                LibTrackPath::new("test/hoge/track1.flac"),
                LibTrackPath::new("test/hoge/track2.flac"),
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
        let mut target = target("test/hoge".to_owned().into(), false, &config, &cui);

        // DB 側から返すパスリストを指定
        target
            .db_track_repository
            .inner
            .expect_get_path_by_path_str()
            .returning(|_| {
                Ok(vec![
                    LibTrackPath::new("test/hoge/child/track1.flac"),
                    LibTrackPath::new("test/hoge/track2.flac"),
                    LibTrackPath::new("test/hoge/db1.flac"),
                ])
            });

        assert_eq!(
            target.listup_track_path(&db_pool).await?,
            vec![
                LibTrackPath::new("test/hoge/child/dap1.flac"),
                LibTrackPath::new("test/hoge/child/pc1.flac"),
                LibTrackPath::new("test/hoge/child/track1.flac"),
                LibTrackPath::new("test/hoge/db1.flac"),
                LibTrackPath::new("test/hoge/pc2.flac"),
                LibTrackPath::new("test/hoge/track2.flac"),
            ]
        );

        checkpoint_all(&mut target);
        Ok(())
    }
}
