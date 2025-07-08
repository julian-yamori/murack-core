use crate::{
    db_wrapper::TransactionWrapper,
    folder::FolderIdMayRoot,
    path::LibSongPath,
    sync::{DbSongSync, SongSync},
};
use anyhow::Result;
use chrono::NaiveDate;
use mockall::automock;

/// PCと連携するための曲データのリポジトリ
#[automock]
pub trait DbSongSyncRepository {
    /// パスを指定して曲情報を取得
    ///
    /// # Arguments
    /// - path 曲のパス
    /// # Returns
    /// 該当する曲の情報（見つからない場合はNone）
    fn get_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibSongPath,
    ) -> Result<Option<DbSongSync>>;

    /// 曲を新規登録
    ///
    /// # Arguments
    /// - song_path: 追加する曲のパス
    /// - song_sync: 登録する曲のデータ
    /// - folder_id: 追加先のライブラリフォルダのID
    /// - entry_date: 登録日
    ///
    /// # Return
    /// 追加した曲のID
    fn register<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_path: &LibSongPath,
        song_sync: &SongSync,
        folder_id: FolderIdMayRoot,
        entry_date: NaiveDate,
    ) -> Result<i32>;

    /// 曲の連携情報をDBに保存(アートワーク以外)
    ///
    /// アートワークは重すぎるので除外。
    /// DbArtworkRepositoryの保存処理を直接呼び出すこと。
    fn save_exclude_artwork<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song: &DbSongSync,
    ) -> Result<()>;
}
