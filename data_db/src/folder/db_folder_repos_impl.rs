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

use crate::converts::enums::db_from_folder_id_may_root;

/// DbFolderRepositoryの本実装
#[derive(new)]
pub struct DbFolderRepositoryImpl {}

#[async_trait]
impl DbFolderRepository for DbFolderRepositoryImpl {
    /// 指定されたフォルダのIDを取得
    async fn get_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Option<i32>> {
        let row = sqlx::query!("SELECT id FROM folder_paths WHERE path = $1", path.as_str())
            .fetch_optional(&mut **tx.get())
            .await?;
        Ok(row.map(|r| r.id))
    }

    /// 指定されたフォルダの、親フォルダのIDを取得
    async fn get_parent<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderIdMayRoot>> {
        let row = sqlx::query!(
            "SELECT parent_id FROM folder_paths WHERE id = $1",
            folder_id
        )
        .fetch_optional(&mut **tx.get())
        .await?;
        Ok(row.map(|r| db_into_folder_id_may_root(r.parent_id)))
    }

    /// 指定されたパスのフォルダが存在するか確認
    async fn is_exist_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<bool> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE path = $1"#,
            path.as_str()
        )
        .fetch_one(&mut **tx.get())
        .await?;
        Ok(count > 0)
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
        let folder_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE parent_id IS NOT DISTINCT FROM $1"#,
            db_from_folder_id_may_root(folder_id)
        )
        .fetch_one(&mut **tx.get())
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
        let existing_id =
            sqlx::query_scalar!("SELECT id FROM folder_paths WHERE path = $1", path.as_str())
                .fetch_optional(&mut **tx.get())
                .await?;

        //見つかった場合はこのIDを返す
        if let Some(i) = existing_id {
            return Ok(FolderIdMayRoot::Folder(i));
        }

        //親ディレクトリについて再帰呼出し、親のID取得
        let parent_id = self
            .register_not_exists(tx, &path.parent().unwrap())
            .await?;

        let my_name = path.dir_name().unwrap();

        let new_id = sqlx::query_scalar!(
            "INSERT INTO folder_paths (path, name, parent_id) VALUES ($1, $2, $3) RETURNING id",
            path.as_str(),
            my_name,
            db_from_folder_id_may_root(parent_id)
        )
        .fetch_one(&mut **tx.get())
        .await?;

        Ok(FolderIdMayRoot::Folder(new_id))
    }

    /// フォルダを削除
    ///
    /// # Arguments
    /// - folder_id: 削除対象のフォルダID
    async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, folder_id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM folder_paths WHERE id = $1", folder_id)
            .execute(&mut **tx.get())
            .await?;
        Ok(())
    }
}
