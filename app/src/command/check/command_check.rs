//! checkコマンド
//!
//! PC・DAP・DBの齟齬を確認する

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use anyhow::Result;
use murack_core_domain::{path::LibraryTrackPath, track::DbTrackRepository};
use sqlx::PgPool;

use super::{
    CommandCheckArgs, ResolveDap, ResolveDataMatch, ResolveExistance, ResolveFileExistanceResult,
};
use crate::{
    Config,
    command::check::domain::{CheckIssueSummary, check_usecase},
    cui::Cui,
};

pub struct CommandCheck<'config, 'cui, CUI, REX, RDM, RDP, SR>
where
    CUI: Cui + Send + Sync,
    REX: ResolveExistance,
    RDM: ResolveDataMatch,
    RDP: ResolveDap,
    SR: DbTrackRepository,
{
    args: CommandCheckArgs,

    resolve_existance: REX,
    resolve_data_match: RDM,
    resolve_dap: RDP,

    config: &'config Config,
    cui: &'cui CUI,
    db_track_repository: SR,
}

impl<'config, 'cui, CUI, REX, RDM, RDP, SR> CommandCheck<'config, 'cui, CUI, REX, RDM, RDP, SR>
where
    CUI: Cui + Send + Sync,
    REX: ResolveExistance,
    RDM: ResolveDataMatch,
    RDP: ResolveDap,
    SR: DbTrackRepository,
{
    pub fn new(
        args: CommandCheckArgs,
        config: &'config Config,

        resolve_existance: REX,
        resolve_data_match: RDM,
        resolve_dap: RDP,
        cui: &'cui CUI,
        db_track_repository: SR,
    ) -> Self {
        Self {
            args,
            resolve_existance,
            resolve_data_match,
            resolve_dap,
            config,
            cui,
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
    async fn listup_track_path(&self, db_pool: &PgPool) -> Result<Vec<LibraryTrackPath>> {
        let cui = &self.cui;

        //マージ用set
        let mut set = BTreeSet::<LibraryTrackPath>::new();

        //PCからリストアップ
        cui_outln!(cui, "PCの検索中...")?;
        let pc_list = match &self.args.path {
            Some(path_str) => {
                murack_core_data_file::search_by_lib_path(&self.config.pc_lib, path_str)?
            }
            None => murack_core_data_file::search_all(&self.config.pc_lib)?,
        };
        for path in pc_list {
            set.insert(path);
        }

        //DAPからリストアップ
        cui_outln!(cui, "DAPの検索中...")?;
        let dap_list = match &self.args.path {
            Some(path_str) => {
                murack_core_data_file::search_by_lib_path(&self.config.dap_lib, path_str)?
            }
            None => murack_core_data_file::search_all(&self.config.dap_lib)?,
        };
        for path in dap_list {
            set.insert(path);
        }

        //DBからリストアップ
        cui_outln!(cui, "DBの検索中...")?;

        let mut tx = db_pool.begin().await?;

        let db_list = match &self.args.path {
            Some(path_str) => {
                self.db_track_repository
                    .get_path_by_path_str(&mut tx, path_str)
                    .await?
            }
            None => self.db_track_repository.get_all_path(&mut tx).await?,
        };
        for path in db_list {
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
        path_list: Vec<LibraryTrackPath>,
    ) -> Result<Vec<LibraryTrackPath>> {
        let cui = &self.cui;

        let mut conflict_list = Vec::<(LibraryTrackPath, Vec<CheckIssueSummary>)>::new();

        //全曲に対して整合性チェック
        let all_count = path_list.len();
        for (current_index, path) in path_list.into_iter().enumerate() {
            if current_index % 100 == 0 {
                cui_outln!(cui, "チェック中...({}/{})", current_index, all_count)?;
            }

            let issues = check_usecase::listup_issue_summary(
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
    fn summary_result_cui(&self, conflict_list: &[LibraryTrackPath]) -> Result<bool> {
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
        conflict_list: &[LibraryTrackPath],
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
    async fn resolve_track(&self, db_pool: &PgPool, track_path: &LibraryTrackPath) -> Result<bool> {
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
