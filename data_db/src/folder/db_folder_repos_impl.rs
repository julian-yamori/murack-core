#[cfg(test)]
mod tests;

use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{
    db::DbTransaction,
    folder::{DbFolderRepository, FolderIdMayRoot},
    path::LibDirPath,
};

use crate::converts::enums::db_into_folder_id_may_root;

use super::FolderPathDao;

/// DbFolderRepositoryの本実装
#[derive(new)]
pub struct DbFolderRepositoryImpl<FPD>
where
    FPD: FolderPathDao + Sync + Send,
{
    folder_path_dao: FPD,
}

#[async_trait]
impl<FPD> DbFolderRepository for DbFolderRepositoryImpl<FPD>
where
    FPD: FolderPathDao + Sync + Send,
{
    /// 指定されたフォルダのIDを取得
    async fn get_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>> {
        self.folder_path_dao.select_id_by_path(tx, path).await
    }

    /// 指定されたフォルダの、親フォルダのIDを取得
    async fn get_parent<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderIdMayRoot>> {
        Ok(self
            .folder_path_dao
            .select_by_id(tx, folder_id)
            .await?
            .map(|f| db_into_folder_id_may_root(f.parent_id)))
    }

    /// 指定されたパスのフォルダが存在するか確認
    async fn is_exist_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<bool> {
        self.folder_path_dao.exists_path(tx, path).await
    }

    /// 指定されたフォルダに、子フォルダが存在するか確認
    ///
    /// folder_idにRootを指定した場合、
    /// ルート直下に子フォルダがあるかを調べる
    async fn is_exist_in_folder<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<bool> {
        let folder_count = self
            .folder_path_dao
            .count_by_parent_id(tx, folder_id)
            .await?;
        Ok(folder_count > 0)
    }

    /// フォルダのパス情報を登録
    ///
    /// 既に同じパスが存在する場合は新規登録せず、IDを返す
    ///
    /// # Arguments
    /// - path: 登録する、ライブラリフォルダ内のパス
    /// # Return
    /// 新規登録されたデータ、もしくは既存のデータのID。
    async fn register_not_exists<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<FolderIdMayRoot> {
        //ライブラリルートならNone
        if path.is_root() {
            return Ok(FolderIdMayRoot::Root);
        }

        //同一パスのデータを検索し、そのIDを取得
        let existing_id = self.folder_path_dao.select_id_by_path(tx, path).await?;

        //見つかった場合はこのIDを返す
        if let Some(i) = existing_id {
            return Ok(FolderIdMayRoot::Folder(i));
        }

        //親ディレクトリについて再帰呼出し、親のID取得
        let parent_id = self
            .register_not_exists(tx, &path.parent().unwrap())
            .await?;

        let my_name = path.dir_name().unwrap();

        let new_id = self
            .folder_path_dao
            .insert(tx, path, my_name, parent_id)
            .await?;

        Ok(FolderIdMayRoot::Folder(new_id))
    }

    /// フォルダを削除
    ///
    /// # Arguments
    /// - folder_id: 削除対象のフォルダID
    async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, folder_id: i32) -> Result<()> {
        self.folder_path_dao.delete_by_id(tx, folder_id).await?;
        Ok(())
    }
}
