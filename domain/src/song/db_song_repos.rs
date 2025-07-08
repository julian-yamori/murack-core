use crate::{
    db_wrapper::TransactionWrapper,
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibPathStr, LibSongPath},
};
use anyhow::Result;
use mockall::automock;

/// 曲データのDBリポジトリ
#[automock]
pub trait DbSongRepository {
    /// パスから曲IDを取得
    fn get_id_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibSongPath,
    ) -> Result<Option<i32>>;

    /// 文字列でパスを指定して、該当曲のパスリストを取得
    fn get_path_by_path_str<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &'c LibPathStr,
    ) -> Result<Vec<LibSongPath>>;

    /// ディレクトリを指定してパスを取得
    /// # Arguments
    /// - path: 検索対象のライブラリパス
    /// # Returns
    /// 指定されたディレクトリ内の、全ての曲のパス
    fn get_path_by_directory<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>>;

    /// ライブラリ内の全ての曲のパスを取得
    fn get_path_all<'c>(&self, tx: &TransactionWrapper<'c>) -> Result<Vec<LibSongPath>>;

    /// 指定したパスの曲が存在するか確認
    fn is_exist_path<'c>(&self, tx: &TransactionWrapper<'c>, path: &LibSongPath) -> Result<bool>;

    /// 指定されたフォルダに曲が存在するか確認
    fn is_exist_in_folder<'c>(&self, tx: &TransactionWrapper<'c>, folder_id: i32) -> Result<bool>;

    /// 曲のパスを書き換え
    ///
    /// # Arguments
    /// - old_path: 書き換え元の曲のパス
    /// - new_path: 書き換え先の曲のパス
    /// - new_folder_id: 新しい親フォルダのID
    fn update_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()>;

    /// 曲の再生時間を書き換え
    fn update_duration<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()>;

    /// 曲を削除
    ///
    /// # Arguments
    /// - song_id: 削除する曲のID
    fn delete<'c>(&self, tx: &TransactionWrapper<'c>, song_id: i32) -> Result<()>;
}
