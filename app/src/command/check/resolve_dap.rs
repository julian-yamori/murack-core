use anyhow::Result;
use mockall::automock;
use murack_core_domain::path::LibraryTrackPath;

use super::messages;
use crate::{
    Config,
    command::check::domain::{CheckIssueSummary, check_usecase},
    cui::Cui,
    data_file,
};

/// PC・DAP間の齟齬の解決処理
#[automock]
pub trait ResolveDap {
    /// PC・DAP間のファイル内容齟齬の解決処理
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    fn resolve_pc_dap_conflict(&self, track_path: &LibraryTrackPath) -> Result<bool>;
}

///ResolveDapの実装
pub struct ResolveDapImpl<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    config: &'config Config,
    cui: &'cui CUI,
}

impl<'config, 'cui, CUI> ResolveDapImpl<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    pub fn new(config: &'config Config, cui: &'cui CUI) -> Self {
        Self { config, cui }
    }
}

impl<'config, 'cui, CUI> ResolveDap for ResolveDapImpl<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    /// PC・DAP間のファイル内容齟齬の解決処理
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    fn resolve_pc_dap_conflict(&self, track_path: &LibraryTrackPath) -> Result<bool> {
        //内容が一致する場合はスキップ
        if check_usecase::check_pc_dap_content(
            &self.config.pc_lib,
            &self.config.dap_lib,
            track_path,
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
            //PCからDAPへ上書き
            '1' => {
                data_file::overwrite_track_over_lib(
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    track_path,
                )?;

                Ok(true)
            }
            '0' => Ok(true),
            '-' => Ok(false),
            _ => unreachable!(),
        }
    }
}
