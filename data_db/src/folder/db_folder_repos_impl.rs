#[cfg(test)]
mod tests;

use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{
    folder::{DbFolderRepository, FolderIdMayRoot},
    path::LibraryDirectoryPath,
};
use sqlx::PgTransaction;

/// DbFolderRepositoryの本実装
#[derive(new)]
pub struct DbFolderRepositoryImpl {}

#[async_trait]
impl DbFolderRepository for DbFolderRepositoryImpl {
    /// 指定されたフォルダのIDを取得
    async fn get_id_by_path<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibraryDirectoryPath,
    ) -> Result<Option<i32>> {
        let row = sqlx::query!(
            "SELECT id FROM folder_paths WHERE path = $1",
            path.as_ref() as &str
        )
        .fetch_optional(&mut **tx)
        .await?;
        Ok(row.map(|r| r.id))
    }

    /// 指定されたフォルダの、親フォルダのIDを取得
    async fn get_parent<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        folder_id: i32,
    ) -> Result<Option<FolderIdMayRoot>> {
        let opt_opt_i = sqlx::query_scalar!(
            "SELECT parent_id FROM folder_paths WHERE id = $1",
            folder_id
        )
        .fetch_optional(&mut **tx)
        .await?;

        Ok(opt_opt_i.map(FolderIdMayRoot::from))
    }

    /// 指定されたパスのフォルダが存在するか確認
    async fn is_exist_path<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibraryDirectoryPath,
    ) -> Result<bool> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE path = $1"#,
            path.as_ref() as &str
        )
        .fetch_one(&mut **tx)
        .await?;
        Ok(count > 0)
    }

    /// 指定されたフォルダに、子フォルダが存在するか確認
    ///
    /// folder_idにRootを指定した場合、
    /// ルート直下に子フォルダがあるかを調べる
    async fn is_exist_in_folder<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        folder_id: FolderIdMayRoot,
    ) -> Result<bool> {
        let folder_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE parent_id IS NOT DISTINCT FROM $1"#,
            folder_id.into_db()
        )
        .fetch_one(&mut **tx)
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
        tx: &mut PgTransaction<'c>,
        path: &LibraryDirectoryPath,
    ) -> Result<i32> {
        //同一パスのデータを検索し、そのIDを取得
        let existing_id = sqlx::query_scalar!(
            "SELECT id FROM folder_paths WHERE path = $1",
            path.as_ref() as &str
        )
        .fetch_optional(&mut **tx)
        .await?;

        //見つかった場合はこのIDを返す
        if let Some(i) = existing_id {
            return Ok(i);
        }

        //親ディレクトリについて再帰呼出し、親のID取得
        let parent_id = match path.parent() {
            Some(parent_path) => {
                let id = self.register_not_exists(tx, &parent_path).await?;
                FolderIdMayRoot::Folder(id)
            }
            None => FolderIdMayRoot::Root,
        };

        let new_id = sqlx::query_scalar!(
            "INSERT INTO folder_paths (path, name, parent_id) VALUES ($1, $2, $3) RETURNING id",
            path.as_ref() as &str,
            path.dir_name(),
            parent_id.into_db()
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(new_id)
    }

    /// フォルダを削除
    ///
    /// # Arguments
    /// - folder_id: 削除対象のフォルダID
    async fn delete<'c>(&self, tx: &mut PgTransaction<'c>, folder_id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM folder_paths WHERE id = $1", folder_id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}
