//! 旧 CheckUsecase
//!
//! 共通関数のような使われ方をしている。

use std::{fs::File, io::prelude::*, path::Path};

use anyhow::Result;
use murack_core_domain::{
    Error, FileLibraryRepository,
    path::LibTrackPath,
    sync::{DbTrackSyncRepository, TrackSync},
    track::TrackItemKind,
};
use sqlx::PgPool;

use super::CheckIssueSummary;

/// 曲のチェックを行い、問題の簡易情報リストを取得
///
/// # Arguments
///
/// ## ignore_dap_content
/// DAPのファイル内容を無視するか。
/// trueなら、PC間とDAP間でファイル内容を比較しない。
/// (一致として扱う)
pub async fn listup_issue_summary<SSR, FLR>(
    db_pool: &PgPool,
    pc_lib: &Path,
    dap_lib: &Path,
    track_path: &LibTrackPath,
    ignore_dap_content: bool,
    db_track_sync_repository: &SSR,
    file_library_repository: &FLR,
) -> Result<Vec<CheckIssueSummary>>
where
    SSR: DbTrackSyncRepository + Sync + Send,
    FLR: FileLibraryRepository,
{
    let mut issue_list = Vec::new();

    //PCデータ読み込み
    let pc_data_opt = match file_library_repository.read_track_sync(pc_lib, track_path) {
        Ok(d) => Some(d),
        Err(e) => match e.downcast_ref() {
            Some(Error::FileTrackNotFound { .. }) => {
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
    let mut tx = db_pool.begin().await?;
    let db_data_opt = db_track_sync_repository
        .get_by_path(&mut tx, track_path)
        .await?;
    tx.commit().await?;

    if db_data_opt.is_none() {
        issue_list.push(CheckIssueSummary::DbNotExists);
    }

    //DAP存在確認
    let dap_exists = track_path.abs(dap_lib).exists();
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
    if !check_editable(&pc_data, &db_data.track_sync).is_empty() {
        issue_list.push(CheckIssueSummary::PcDbNotEqualsEditable);
    }
    if !check_duration(&pc_data, &db_data.track_sync) {
        issue_list.push(CheckIssueSummary::PcDbNotEqualsDuration);
    }
    if !check_artwork(&pc_data, &db_data.track_sync) {
        issue_list.push(CheckIssueSummary::PcDbNotEqualsArtwork);
    }

    //PCとDAPの比較(無視指定されていない場合のみ)
    if !ignore_dap_content && !check_pc_dap_content(pc_lib, dap_lib, track_path)? {
        issue_list.push(CheckIssueSummary::PcDapNotEquals);
    }

    Ok(issue_list)
}

/// PCとDBで曲データの編集可能部分を比較
/// # Returns
/// 不一致の項目のリスト
/// 全て一致したら空Vec。
pub fn check_editable(pc_data: &TrackSync, db_data: &TrackSync) -> Vec<TrackItemKind> {
    let mut conflicts = Vec::new();

    if pc_data.title != db_data.title {
        conflicts.push(TrackItemKind::Title);
    }
    if pc_data.artist != db_data.artist {
        conflicts.push(TrackItemKind::Artist);
    }
    if pc_data.album != db_data.album {
        conflicts.push(TrackItemKind::Album);
    }
    if pc_data.genre != db_data.genre {
        conflicts.push(TrackItemKind::Genre);
    }
    if pc_data.album_artist != db_data.album_artist {
        conflicts.push(TrackItemKind::AlbumArtist);
    }
    if pc_data.composer != db_data.composer {
        conflicts.push(TrackItemKind::Composer);
    }
    if pc_data.track_number != db_data.track_number {
        conflicts.push(TrackItemKind::TrackNumber);
    }
    if pc_data.track_max != db_data.track_max {
        conflicts.push(TrackItemKind::TrackMax);
    }
    if pc_data.disc_number != db_data.disc_number {
        conflicts.push(TrackItemKind::DiscNumber);
    }
    if pc_data.disc_max != db_data.disc_max {
        conflicts.push(TrackItemKind::DiscMax);
    }
    if pc_data.release_date != db_data.release_date {
        conflicts.push(TrackItemKind::ReleaseDate);
    }
    if pc_data.memo != db_data.memo {
        conflicts.push(TrackItemKind::Memo);
    }
    if pc_data.lyrics != db_data.lyrics {
        conflicts.push(TrackItemKind::Lyrics);
    }

    conflicts
}

/// PCとDBの再生時間を比較
/// #Returns
/// 一致したらtrue
pub fn check_duration(pc_data: &TrackSync, db_data: &TrackSync) -> bool {
    pc_data.duration == db_data.duration
}

/// PCとDBのアートワークを比較
/// #Returns
/// 一致したらtrue
pub fn check_artwork(pc_data: &TrackSync, db_data: &TrackSync) -> bool {
    pc_data.artworks == db_data.artworks
}

/// PCとDAPのファイル内容を比較
/// # Returns
/// 差異がない場合はtrue
pub fn check_pc_dap_content(
    pc_lib: &Path,
    dap_lib: &Path,
    track_path: &LibTrackPath,
) -> Result<bool> {
    //PCデータ読み込み
    let pc_path = track_path.abs(pc_lib);
    let mut pc_file =
        File::open(&pc_path).map_err(|e| Error::FileIoError(pc_path.to_owned(), e))?;
    let mut pc_content = Vec::new();
    pc_file
        .read_to_end(&mut pc_content)
        .map_err(|e| Error::FileIoError(pc_path, e))?;

    //DAPデータ読み込み
    let dap_path = track_path.abs(dap_lib);
    let mut dap_file =
        File::open(&dap_path).map_err(|e| Error::FileIoError(dap_path.to_owned(), e))?;
    let mut dap_content = Vec::new();
    dap_file
        .read_to_end(&mut dap_content)
        .map_err(|e| Error::FileIoError(dap_path, e))?;

    Ok(pc_content == dap_content)
}
