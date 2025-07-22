use anyhow::Result;
use mockall::automock;
use murack_core_domain::{
    FileLibraryRepository,
    check::{CheckIssueSummary, CheckUsecase},
    path::LibSongPath,
};

use super::messages;
use crate::{Config, cui::Cui};

/// PC・DAP間の齟齬の解決処理
#[automock]
pub trait ResolveDap {
    /// PC・DAP間のファイル内容齟齬の解決処理
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    fn resolve_pc_dap_conflict(&self, song_path: &LibSongPath) -> Result<bool>;
}

///ResolveDapの実装
pub struct ResolveDapImpl<'config, 'cui, CUI, FR, CS>
where
    CUI: Cui + Send + Sync,
    FR: FileLibraryRepository,
    CS: CheckUsecase,
{
    config: &'config Config,
    cui: &'cui CUI,
    file_library_repository: FR,
    check_usecase: CS,
}

impl<'config, 'cui, CUI, FR, CS> ResolveDapImpl<'config, 'cui, CUI, FR, CS>
where
    CUI: Cui + Send + Sync,
    FR: FileLibraryRepository,
    CS: CheckUsecase,
{
    pub fn new(
        config: &'config Config,
        cui: &'cui CUI,
        file_library_repository: FR,
        check_usecase: CS,
    ) -> Self {
        Self {
            config,
            cui,
            file_library_repository,
            check_usecase,
        }
    }
}

impl<'config, 'cui, CUI, FR, CS> ResolveDap for ResolveDapImpl<'config, 'cui, CUI, FR, CS>
where
    CUI: Cui + Send + Sync,
    FR: FileLibraryRepository,
    CS: CheckUsecase,
{
    /// PC・DAP間のファイル内容齟齬の解決処理
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    fn resolve_pc_dap_conflict(&self, song_path: &LibSongPath) -> Result<bool> {
        //内容が一致する場合はスキップ
        if self.check_usecase.check_pc_dap_content(
            &self.config.pc_lib,
            &self.config.dap_lib,
            song_path,
        )? {
            return Ok(true);
        }

        let cui = &self.cui;
        cui_outln!(cui, "----")?;
        cui_outln!(cui, "{}", CheckIssueSummary::PcDapNotEquals)?;
        cui_outln!(cui)?;

        cui_outln!(cui, "1: PCからDAPへファイルを上書き")?;
        cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
        cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
        cui_outln!(cui)?;

        let input = cui.input_case(&['1', '0', '-'], messages::MSG_SELECT_OPERATION)?;

        match input {
            //PCからDBへ上書き
            '1' => {
                self.file_library_repository.overwrite_song_over_lib(
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    song_path,
                )?;

                Ok(true)
            }
            '0' => Ok(true),
            '-' => Ok(false),
            _ => unreachable!(),
        }
    }
}
