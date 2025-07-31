#[cfg(test)]
mod tests;

use anyhow::Result;
use async_recursion::async_recursion;
use sqlx::PgTransaction;

use crate::{
    folder::{FolderIdMayRoot, FolderPathError},
    path::LibraryDirectoryPath,
};

/// 指定されたパスのフォルダが存在するか確認
pub async fn is_exist_path<'c>(
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

/// フォルダのパス情報を登録
///
/// 既に同じパスが存在する場合は新規登録せず、IDを返す
///
/// # Arguments
/// - path: 登録する、ライブラリフォルダ内のパス
/// # Return
/// 新規登録されたデータ、もしくは既存のデータのID。
#[async_recursion]
pub async fn register_not_exists<'c>(
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
            let id = register_not_exists(tx, &parent_path).await?;
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

/// フォルダに曲が含まれてない場合、削除する
///
/// # Arguments
/// - folder_path: 確認・削除対象のフォルダパス
pub async fn delete_if_empty<'c>(
    tx: &mut PgTransaction<'c>,
    folder_path: &LibraryDirectoryPath,
) -> Result<()> {
    let folder_id_opt = sqlx::query_scalar!(
        "SELECT id FROM folder_paths WHERE path = $1",
        folder_path.as_ref() as &str
    )
    .fetch_optional(&mut **tx)
    .await?;

    //IDを取得
    let folder_id = folder_id_opt
        .ok_or_else(|| FolderPathError::DbFolderPathNotFound(folder_path.to_owned()))?;

    delete_if_empty_by_id(tx, folder_id).await
}

/// フォルダに曲が含まれてない場合、削除する(再帰実行用のID指定版)
///
/// # Arguments
/// - folder_path: 確認・削除対象のフォルダパス
#[async_recursion]
async fn delete_if_empty_by_id<'c>(tx: &mut PgTransaction<'c>, folder_id: i32) -> Result<()> {
    //他の曲が含まれる場合、削除せずに終了
    let track_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE folder_id = $1"#,
        folder_id,
    )
    .fetch_one(&mut **tx)
    .await?;
    if track_count > 0 {
        return Ok(());
    }

    let parent_id_mr = {
        //他のフォルダが含まれる場合、削除せずに終了
        let other_folder_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM folder_paths WHERE parent_id = $1"#,
            folder_id
        )
        .fetch_one(&mut **tx)
        .await?;
        if other_folder_count > 0 {
            return Ok(());
        }

        //削除するフォルダ情報を取得
        let opt_mr = sqlx::query_scalar!(
            "SELECT parent_id FROM folder_paths WHERE id = $1",
            folder_id
        )
        .fetch_optional(&mut **tx)
        .await?
        .map(FolderIdMayRoot::from);
        let parent_id_mr = opt_mr.ok_or(FolderPathError::DbFolderIdNotFound(folder_id))?;

        //削除を実行
        sqlx::query!("DELETE FROM folder_paths WHERE id = $1", folder_id)
            .execute(&mut **tx)
            .await?;

        parent_id_mr
    };

    //親フォルダについて再帰実行
    if let FolderIdMayRoot::Folder(parent_id) = parent_id_mr {
        delete_if_empty_by_id(tx, parent_id).await?;
    }

    Ok(())
}
