use super::CheckIssueSummary;
use crate::{
    db_wrapper::ConnectionWrapper,
    path::LibSongPath,
    song::SongItemKind,
    sync::{DbSongSyncRepository, SongSync},
    Error, FileLibraryRepository,
};
use anyhow::Result;
use mockall::automock;
use std::rc::Rc;
use std::{fs::File, io::prelude::*, path::Path};

/// PC・DB・DAP間の曲の整合性チェックのUsecase
#[automock]
pub trait CheckUsecase {
    /// 曲のチェックを行い、問題の簡易情報リストを取得
    ///
    /// # Arguments
    ///
    /// ## ignore_dap_content
    /// DAPのファイル内容を無視するか。
    /// trueなら、PC間とDAP間でファイル内容を比較しない。
    /// (一致として扱う)
    fn listup_issue_summary(
        &self,
        db: &mut ConnectionWrapper,
        pc_lib: &Path,
        dap_lib: &Path,
        song_path: &LibSongPath,
        ignore_dap_content: bool,
    ) -> Result<Vec<CheckIssueSummary>>;

    /// PCとDBで曲データの編集可能部分を比較
    /// # Returns
    /// 不一致の項目のリスト
    /// 全て一致したら空Vec。
    fn check_editable(&self, pc_data: &SongSync, db_data: &SongSync) -> Vec<SongItemKind>;

    /// PCとDBの再生時間を比較
    /// #Returns
    /// 一致したらtrue
    fn check_duration(&self, pc_data: &SongSync, db_data: &SongSync) -> bool;

    /// PCとDBのアートワークを比較
    /// #Returns
    /// 一致したらtrue
    fn check_artwork(&self, pc_data: &SongSync, db_data: &SongSync) -> bool;

    /// PCとDAPのファイル内容を比較
    /// # Returns
    /// 差異がない場合はtrue
    fn check_pc_dap_content(
        &self,
        pc_lib: &Path,
        dap_lib: &Path,
        song_path: &LibSongPath,
    ) -> Result<bool>;
}

/// CheckUsecaseの本実装
#[derive(new)]
pub struct CheckUsecaseImpl {
    db_song_sync_repository: Rc<dyn DbSongSyncRepository>,
    file_library_repository: Rc<dyn FileLibraryRepository>,
}

impl CheckUsecase for CheckUsecaseImpl {
    /// 曲のチェックを行い、問題の簡易情報リストを取得
    ///
    /// # Arguments
    ///
    /// ## ignore_dap_content
    /// DAPのファイル内容を無視するか。
    /// trueなら、PC間とDAP間でファイル内容を比較しない。
    /// (一致として扱う)
    fn listup_issue_summary(
        &self,
        db: &mut ConnectionWrapper,
        pc_lib: &Path,
        dap_lib: &Path,
        song_path: &LibSongPath,
        ignore_dap_content: bool,
    ) -> Result<Vec<CheckIssueSummary>> {
        let mut issue_list = Vec::new();

        //PCデータ読み込み
        let pc_data_opt = match self
            .file_library_repository
            .read_song_sync(pc_lib, song_path)
        {
            Ok(d) => Some(d),
            Err(e) => match e.downcast_ref() {
                Some(Error::FileSongNotFound { .. }) => {
                    issue_list.push(CheckIssueSummary::PcNotExists);
                    None
                }
                _ => {
                    issue_list.push(CheckIssueSummary::PcReadFailed { e });
                    None
                }
            },
        };

        //DBデータ読み込み
        let db_data_opt =
            db.run_in_transaction(|tx| self.db_song_sync_repository.get_by_path(tx, song_path))?;
        if db_data_opt.is_none() {
            issue_list.push(CheckIssueSummary::DbNotExists);
        }

        //DAP存在確認
        let dap_exists = song_path.abs(dap_lib).exists();
        if !dap_exists {
            issue_list.push(CheckIssueSummary::DapNotExists);

            //1つでも取得できない箇所があれば、以降のチェックは行わない
            return Ok(issue_list);
        }

        //1つでも取得できない箇所があれば、以降のチェックは行わない
        let (pc_data, db_data) = match (pc_data_opt, db_data_opt) {
            (Some(p), Some(d)) => (p, d),
            (_, _) => return Ok(issue_list),
        };

        //PCとDBのデータ比較比較
        if !self.check_editable(&pc_data, &db_data.song_sync).is_empty() {
            issue_list.push(CheckIssueSummary::PcDbNotEqualsEditable);
        }
        if !self.check_duration(&pc_data, &db_data.song_sync) {
            issue_list.push(CheckIssueSummary::PcDbNotEqualsDuration);
        }
        if !self.check_artwork(&pc_data, &db_data.song_sync) {
            issue_list.push(CheckIssueSummary::PcDbNotEqualsArtwork);
        }

        //PCとDAPの比較(無視指定されていない場合のみ)
        if !ignore_dap_content && !self.check_pc_dap_content(pc_lib, dap_lib, song_path)? {
            issue_list.push(CheckIssueSummary::PcDapNotEquals);
        }

        Ok(issue_list)
    }

    /// PCとDBで曲データの編集可能部分を比較
    /// # Returns
    /// 不一致の項目のリスト
    /// 全て一致したら空Vec。
    fn check_editable(&self, pc_data: &SongSync, db_data: &SongSync) -> Vec<SongItemKind> {
        let mut conflicts = Vec::new();

        if pc_data.title != db_data.title {
            conflicts.push(SongItemKind::Title);
        }
        if pc_data.artist != db_data.artist {
            conflicts.push(SongItemKind::Artist);
        }
        if pc_data.album != db_data.album {
            conflicts.push(SongItemKind::Album);
        }
        if pc_data.genre != db_data.genre {
            conflicts.push(SongItemKind::Genre);
        }
        if pc_data.album_artist != db_data.album_artist {
            conflicts.push(SongItemKind::AlbumArtist);
        }
        if pc_data.composer != db_data.composer {
            conflicts.push(SongItemKind::Composer);
        }
        if pc_data.track_number != db_data.track_number {
            conflicts.push(SongItemKind::TrackNumber);
        }
        if pc_data.track_max != db_data.track_max {
            conflicts.push(SongItemKind::TrackMax);
        }
        if pc_data.disc_number != db_data.disc_number {
            conflicts.push(SongItemKind::DiscNumber);
        }
        if pc_data.disc_max != db_data.disc_max {
            conflicts.push(SongItemKind::DiscMax);
        }
        if pc_data.release_date != db_data.release_date {
            conflicts.push(SongItemKind::ReleaseDate);
        }
        if pc_data.memo != db_data.memo {
            conflicts.push(SongItemKind::Memo);
        }
        if pc_data.lyrics != db_data.lyrics {
            conflicts.push(SongItemKind::Lyrics);
        }

        conflicts
    }

    /// PCとDBの再生時間を比較
    /// #Returns
    /// 一致したらtrue
    fn check_duration(&self, pc_data: &SongSync, db_data: &SongSync) -> bool {
        pc_data.duration == db_data.duration
    }

    /// PCとDBのアートワークを比較
    /// #Returns
    /// 一致したらtrue
    fn check_artwork(&self, pc_data: &SongSync, db_data: &SongSync) -> bool {
        pc_data.artworks == db_data.artworks
    }

    /// PCとDAPのファイル内容を比較
    /// # Returns
    /// 差異がない場合はtrue
    fn check_pc_dap_content(
        &self,
        pc_lib: &Path,
        dap_lib: &Path,
        song_path: &LibSongPath,
    ) -> Result<bool> {
        //PCデータ読み込み
        let pc_path = song_path.abs(pc_lib);
        let mut pc_file =
            File::open(&pc_path).map_err(|e| Error::FileIoError(pc_path.to_owned(), e))?;
        let mut pc_content = Vec::new();
        pc_file
            .read_to_end(&mut pc_content)
            .map_err(|e| Error::FileIoError(pc_path, e))?;

        //DAPデータ読み込み
        let dap_path = song_path.abs(dap_lib);
        let mut dap_file =
            File::open(&dap_path).map_err(|e| Error::FileIoError(dap_path.to_owned(), e))?;
        let mut dap_content = Vec::new();
        dap_file
            .read_to_end(&mut dap_content)
            .map_err(|e| Error::FileIoError(dap_path, e))?;

        Ok(pc_content == dap_content)
    }
}
