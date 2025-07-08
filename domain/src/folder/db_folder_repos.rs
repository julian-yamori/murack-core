use crate::{db_wrapper::TransactionWrapper, folder::FolderIdMayRoot, path::LibDirPath};
use anyhow::Result;
use mockall::automock;

/// フォルダ関係のDBリポジトリ
#[automock]
pub trait DbFolderRepository {
    /// 指定されたフォルダのIDを取得
    fn get_id_by_path<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>>;

    /// 指定されたフォルダの、親フォルダのIDを取得
    fn get_parent<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderIdMayRoot>>;

    /// 指定されたパスのフォルダが存在するか確認
    fn is_exist_path<'c>(&self, tx: &TransactionWrapper<'c>, path: &LibDirPath) -> Result<bool>;

    /// 指定されたフォルダに、子フォルダが存在するか確認
    ///
    /// folder_idにRootを指定した場合、
    /// ルート直下に子フォルダがあるかを調べる
    fn is_exist_in_folder<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<bool>;

    /// フォルダのパス情報を登録
    ///
    /// 既に同じパスが存在する場合は新規登録せず、IDを返す。
    /// pathがrootの場合も登録せず、FolderIdMayRoot::Rootを返す。
    ///
    /// # Arguments
    /// - path: 登録する、ライブラリフォルダ内のパス
    /// # Return
    /// 新規登録されたデータ、もしくは既存のデータのID。
    fn register_not_exists<'c>(
        &self,
        tx: &TransactionWrapper<'c>,
        path: &LibDirPath,
    ) -> Result<FolderIdMayRoot>;

    /// フォルダを削除
    ///
    /// # Arguments
    /// - folder_id: 削除対象のフォルダID
    fn delete<'c>(&self, tx: &TransactionWrapper<'c>, folder_id: i32) -> Result<()>;
}
