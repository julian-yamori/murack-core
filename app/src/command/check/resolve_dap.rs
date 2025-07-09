use super::messages;
use crate::{AppComponents, Config, cui::Cui};
use anyhow::Result;
use domain::{
    FileLibraryRepository,
    check::{CheckIssueSummary, CheckUsecase},
    path::LibSongPath,
};
use mockall::automock;
use std::rc::Rc;

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
pub struct ResolveDapImpl {
    config: Rc<Config>,
    cui: Rc<dyn Cui>,
    file_library_repository: Rc<dyn FileLibraryRepository>,
    check_usecase: Rc<dyn CheckUsecase>,
}

impl ResolveDapImpl {
    pub fn new(app_components: &impl AppComponents) -> Self {
        Self {
            config: app_components.config().clone(),
            cui: app_components.cui().clone(),
            file_library_repository: app_components.file_library_repository().clone(),
            check_usecase: app_components.check_usecase().clone(),
        }
    }
}

impl ResolveDap for ResolveDapImpl {
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
        cui_outln!(cui, "----");
        cui_outln!(cui, "{}", CheckIssueSummary::PcDapNotEquals);
        cui_outln!(cui);

        cui_outln!(cui, "1: PCからDAPへファイルを上書き");
        cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE);
        cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE);
        cui_outln!(cui);

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
