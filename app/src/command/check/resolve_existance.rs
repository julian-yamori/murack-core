use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;
use murack_core_domain::{Error as DomainError, path::LibraryTrackPath};
use sqlx::PgPool;

use super::{ResolveFileExistanceResult, messages};
use crate::{
    Config,
    command::check::domain::CheckIssueSummary,
    cui::Cui,
    data_file, db_common,
    track_sync::{TrackSync, track_sync_repository},
};

/// データ存在系の解決処理
#[automock]
#[async_trait]
pub trait ResolveExistance {
    /// データ存在系の解決処理
    ///
    /// # Arguments
    /// - config: アプリの設定情報
    /// - args: checkコマンドの引数
    /// - cui: CUI実装
    /// - track_path: 作業対象の曲のパス
    async fn resolve(
        &self,
        db_pool: &PgPool,
        track_path: &LibraryTrackPath,
    ) -> Result<ResolveFileExistanceResult>;
}

/// ResolveExistanceの実装
pub struct ResolveExistanceImpl<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    config: &'config Config,
    cui: &'cui CUI,
}

#[async_trait]
impl<'config, 'cui, CUI> ResolveExistance for ResolveExistanceImpl<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    /// データ存在系の解決処理
    ///
    /// # Arguments
    /// - config: アプリの設定情報
    /// - args: checkコマンドの引数
    /// - cui: CUI実装
    /// - track_path: 作業対象の曲のパス
    async fn resolve(
        &self,
        db_pool: &PgPool,
        track_path: &LibraryTrackPath,
    ) -> Result<ResolveFileExistanceResult> {
        //PCデータ読み込み
        let pc_read_result = data_file::read_track_sync(&self.config.pc_lib, track_path);
        let pc_data_opt = match pc_read_result {
            Ok(d) => Some(d),
            Err(e) => match e.downcast_ref() {
                Some(DomainError::FileTrackNotFound { .. }) => None,
                _ => {
                    //読み込み失敗の場合は専用の解決処理
                    //実態は通知のみで、
                    //次のファイルに移動するか、中止するかの選択
                    let r = match self.resole_pc_read_failed(e)? {
                        true => ResolveFileExistanceResult::UnResolved,
                        false => ResolveFileExistanceResult::Terminated,
                    };
                    return Ok(r);
                }
            },
        };

        //DBデータ読み込み
        let db_data_opt = {
            let mut tx = db_pool.begin().await?;

            track_sync_repository::get_by_path(&mut tx, track_path).await?
        };

        //DAP存在確認
        let dap_exists = track_path.abs(&self.config.dap_lib).exists();

        let pc_exists = pc_data_opt.is_some();
        let db_exists = db_data_opt.is_some();

        let result = if pc_exists && db_exists && !dap_exists {
            self.resolve_not_exists_dap(track_path)?
        } else if pc_exists && !db_exists && dap_exists {
            self.resolve_not_exists_db(db_pool, track_path, &mut pc_data_opt.unwrap())
                .await?
        } else if pc_exists && !db_exists && !dap_exists {
            self.resolve_not_exists_db_dap(db_pool, track_path, &mut pc_data_opt.unwrap())
                .await?
        } else if !pc_exists && db_exists && dap_exists {
            self.resolve_not_exists_pc(db_pool, track_path).await?
        } else if !pc_exists && db_exists && !dap_exists {
            self.resolve_not_exists_pc_dap(db_pool, track_path).await?
        } else if !pc_exists && !db_exists && dap_exists {
            self.resolve_not_exists_pc_db(db_pool, track_path).await?
        } else {
            //問題なし
            ResolveFileExistanceResult::Resolved
        };

        Ok(result)
    }
}

impl<'config, 'cui, CUI> ResolveExistanceImpl<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    pub fn new(config: &'config Config, cui: &'cui CUI) -> Self {
        Self { config, cui }
    }

    /// PCから読み込めない状態の解決
    ///
    /// 実際には解決不能なので、
    /// 次のファイルに移動するか、全体の中止かを選ぶのみ
    ///
    /// # Arguments
    /// - read_error: ファイル読み込み時に発生したエラー
    ///
    /// # Returns
    /// 全ファイルの解決処理を続行するか
    fn resole_pc_read_failed(&self, read_error: anyhow::Error) -> Result<bool> {
        let cui = &self.cui;

        cui_outln!(cui, "----")?;
        cui_outln!(cui, "{}", CheckIssueSummary::PcReadFailed { e: read_error })?;
        cui_outln!(cui)?;

        cui_outln!(cui, "{}", messages::CASE_MSG_CANT_RESOLVE)?;
        cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
        cui_outln!(cui)?;

        let input = cui.input_case(&['0', '-'], messages::MSG_SELECT_OPERATION)?;

        Ok(input != '-')
    }

    /// DAPにのみ存在しない状態の解決
    fn resolve_not_exists_dap(
        &self,
        track_path: &LibraryTrackPath,
    ) -> Result<ResolveFileExistanceResult> {
        let cui = &self.cui;

        cui_outln!(cui, "----")?;
        cui_outln!(cui, "{}", CheckIssueSummary::DapNotExists)?;
        cui_outln!(cui)?;

        cui_outln!(cui, "1: PCからDAPへコピー")?;
        cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
        cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
        cui_outln!(cui)?;

        let input = cui.input_case(&['1', '0', '-'], messages::MSG_SELECT_OPERATION)?;

        match input {
            //PCからDAPへコピー
            '1' => {
                data_file::copy_track_over_lib(
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    track_path,
                )?;
                Ok(ResolveFileExistanceResult::Resolved)
            }
            '0' => Ok(ResolveFileExistanceResult::UnResolved),
            '-' => Ok(ResolveFileExistanceResult::Terminated),
            _ => unreachable!(),
        }
    }

    /// DBにのみ存在しない状態の解決
    async fn resolve_not_exists_db(
        &self,
        db_pool: &PgPool,
        track_path: &LibraryTrackPath,
        pc_track: &mut TrackSync,
    ) -> Result<ResolveFileExistanceResult> {
        let input = {
            let cui = &self.cui;

            cui_outln!(cui, "----")?;
            cui_outln!(cui, "{}", CheckIssueSummary::DbNotExists)?;
            cui_outln!(cui)?;

            cui_outln!(cui, "1: DBに曲を追加")?;
            cui_outln!(cui, "2: PCとDAPからファイルを削除")?;
            cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
            cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
            cui_outln!(cui)?;

            cui.input_case(&['1', '2', '0', '-'], messages::MSG_SELECT_OPERATION)?
        };

        match input {
            //DBに曲を追加
            '1' => {
                db_common::add_track_to_db(db_pool, track_path, pc_track).await?;
                Ok(ResolveFileExistanceResult::Resolved)
            }
            //PCとDAPからファイルを削除
            '2' => {
                data_file::trash_track(&self.config.pc_lib, track_path)?;
                data_file::delete_track(&self.config.dap_lib, track_path)?;
                Ok(ResolveFileExistanceResult::Deleted)
            }
            '0' => Ok(ResolveFileExistanceResult::UnResolved),
            '-' => Ok(ResolveFileExistanceResult::Terminated),
            _ => unreachable!(),
        }
    }

    /// DBとDAPに存在しない状態の解決
    async fn resolve_not_exists_db_dap(
        &self,
        db_pool: &PgPool,
        track_path: &LibraryTrackPath,
        pc_track: &mut TrackSync,
    ) -> Result<ResolveFileExistanceResult> {
        let input = {
            let cui = &self.cui;

            cui_outln!(cui, "----")?;
            cui_outln!(cui, "{}", CheckIssueSummary::DbNotExists)?;
            cui_outln!(cui, "{}", CheckIssueSummary::DapNotExists)?;
            cui_outln!(cui)?;

            cui_outln!(cui, "1: DBに曲を追加し、DAPにもコピー")?;
            cui_outln!(cui, "2: PCからファイルを削除")?;
            cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
            cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
            cui_outln!(cui)?;

            cui.input_case(&['1', '2', '0', '-'], messages::MSG_SELECT_OPERATION)?
        };

        match input {
            //DBに曲を追加し、DAPにもコピー
            '1' => {
                db_common::add_track_to_db(db_pool, track_path, pc_track).await?;

                data_file::copy_track_over_lib(
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    track_path,
                )?;

                Ok(ResolveFileExistanceResult::Resolved)
            }
            //PCからファイルを削除
            '2' => {
                data_file::trash_track(&self.config.pc_lib, track_path)?;

                Ok(ResolveFileExistanceResult::Deleted)
            }
            '0' => Ok(ResolveFileExistanceResult::UnResolved),
            '-' => Ok(ResolveFileExistanceResult::Terminated),
            _ => unreachable!(),
        }
    }

    /// PCにのみ存在しない状態の解決
    async fn resolve_not_exists_pc(
        &self,
        db_pool: &PgPool,
        track_path: &LibraryTrackPath,
    ) -> Result<ResolveFileExistanceResult> {
        let input = {
            let cui = &self.cui;

            cui_outln!(cui, "----")?;
            cui_outln!(cui, "{}", CheckIssueSummary::PcNotExists)?;
            cui_outln!(cui)?;

            cui_outln!(cui, "1: DAPからPCにファイルをコピー")?;
            cui_outln!(cui, "2: DBとDAPから曲を削除")?;
            cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
            cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
            cui_outln!(cui)?;

            cui.input_case(&['1', '2', '0', '-'], messages::MSG_SELECT_OPERATION)?
        };

        match input {
            //DAPからPCにファイルをコピー
            '1' => {
                data_file::copy_track_over_lib(
                    &self.config.dap_lib,
                    &self.config.pc_lib,
                    track_path,
                )?;

                Ok(ResolveFileExistanceResult::Resolved)
            }
            //DBとDAPから曲を削除
            '2' => {
                self.delete_track_db(db_pool, track_path).await?;

                data_file::delete_track(&self.config.dap_lib, track_path)?;

                Ok(ResolveFileExistanceResult::Deleted)
            }
            '0' => Ok(ResolveFileExistanceResult::UnResolved),
            '-' => Ok(ResolveFileExistanceResult::Terminated),
            _ => unreachable!(),
        }
    }

    /// PCとDAPに存在しない状態の解決
    async fn resolve_not_exists_pc_dap(
        &self,
        db_pool: &PgPool,
        track_path: &LibraryTrackPath,
    ) -> Result<ResolveFileExistanceResult> {
        let input = {
            let cui = &self.cui;

            cui_outln!(cui, "----")?;
            cui_outln!(cui, "{}", CheckIssueSummary::PcNotExists)?;
            cui_outln!(cui, "{}", CheckIssueSummary::DapNotExists)?;
            cui_outln!(cui)?;

            cui_outln!(cui, "2: DBから曲を削除")?;
            cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
            cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
            cui_outln!(cui)?;

            cui.input_case(&['2', '0', '-'], messages::MSG_SELECT_OPERATION)?
        };

        match input {
            //DBから曲を削除
            '2' => {
                self.delete_track_db(db_pool, track_path).await?;
                Ok(ResolveFileExistanceResult::Deleted)
            }
            '0' => Ok(ResolveFileExistanceResult::UnResolved),
            '-' => Ok(ResolveFileExistanceResult::Terminated),
            _ => unreachable!(),
        }
    }

    /// PCとDBに存在しない状態の解決
    async fn resolve_not_exists_pc_db(
        &self,
        db_pool: &PgPool,
        track_path: &LibraryTrackPath,
    ) -> Result<ResolveFileExistanceResult> {
        let cui = &self.cui;

        cui_outln!(cui, "----")?;
        cui_outln!(cui, "{}", CheckIssueSummary::PcNotExists)?;
        cui_outln!(cui, "{}", CheckIssueSummary::DbNotExists)?;
        cui_outln!(cui)?;

        cui_outln!(cui, "1: DAPからPCにファイルをコピーし、DBにも追加")?;
        cui_outln!(cui, "2: DAPからファイルを削除")?;
        cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
        cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
        cui_outln!(cui)?;

        let input = cui.input_case(&['1', '2', '0', '-'], messages::MSG_SELECT_OPERATION)?;

        match input {
            //DAPからPCにファイルをコピーし、DBにも追加
            '1' => {
                //DAPからPCにコピー
                data_file::copy_track_over_lib(
                    &self.config.dap_lib,
                    &self.config.pc_lib,
                    track_path,
                )?;

                //DAPからコピーしたPCデータを読み込む
                let mut pc_track = match data_file::read_track_sync(&self.config.pc_lib, track_path)
                {
                    Ok(d) => d,
                    Err(e) => {
                        cui_outln!(cui, "曲ファイルのデータの読み込みに失敗しました。\n{}", e)?;
                        return Ok(ResolveFileExistanceResult::UnResolved);
                    }
                };

                //DBに追加
                db_common::add_track_to_db(db_pool, track_path, &mut pc_track).await?;
                Ok(ResolveFileExistanceResult::Resolved)
            }
            //DAPからファイルを削除
            '2' => {
                data_file::delete_track(&self.config.dap_lib, track_path)?;
                Ok(ResolveFileExistanceResult::Deleted)
            }
            '0' => Ok(ResolveFileExistanceResult::UnResolved),
            '-' => Ok(ResolveFileExistanceResult::Terminated),
            _ => unreachable!(),
        }
    }

    /// DBから曲を削除
    async fn delete_track_db(&self, db_pool: &PgPool, track_path: &LibraryTrackPath) -> Result<()> {
        let mut tx = db_pool.begin().await?;

        db_common::delete_track_db(&mut tx, track_path).await?;

        tx.commit().await?;
        Ok(())
    }
}
