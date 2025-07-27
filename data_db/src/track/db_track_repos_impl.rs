use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibPathStr, LibTrackPath},
    track::DbTrackRepository,
};
use sqlx::PgTransaction;

use crate::like_esc;

use super::track_sqls;

/// HasDbTrackRepositoryの本実装
#[derive(new)]
pub struct DbTrackRepositoryImpl {}

#[async_trait]
impl DbTrackRepository for DbTrackRepositoryImpl {
    /// パスから曲IDを取得
    async fn get_id_by_path<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<Option<i32>> {
        let id = sqlx::query_scalar!(
            "SELECT id FROM tracks WHERE path = $1",
            path.as_ref() as &str
        )
        .fetch_optional(&mut **tx)
        .await?;

        Ok(id)
    }

    /// 文字列でパスを指定して、該当曲のパスリストを取得
    async fn get_path_by_path_str<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibPathStr,
    ) -> Result<Vec<LibTrackPath>> {
        //ディレクトリ指定とみなして検索
        let dir_path = path.to_dir_path();
        let mut list = self.get_path_by_directory(tx, &dir_path).await?;

        //ファイル指定とみなしての検索でヒットしたら追加
        if let Some(track_path) = self.path_str_as_track_path(tx, path).await? {
            list.push(track_path);
        }

        Ok(list)
    }

    /// ディレクトリを指定してパスを取得
    /// # Arguments
    /// - path: 検索対象のライブラリパス
    /// # Returns
    /// 指定されたディレクトリ内の、全ての曲のパス
    async fn get_path_by_directory<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibTrackPath>> {
        if path.is_root() {
            //ルートフォルダ指定なら、全曲
            let paths = sqlx::query_scalar!(r#"SELECT path AS "path: LibTrackPath" FROM tracks"#)
                .fetch_all(&mut **tx)
                .await?;
            Ok(paths)
        } else {
            let path_str = path.as_str();

            //LIKE文エスケープ
            let cmp_value_buff;
            let (like_query, cmp_value) = if like_esc::is_need(path_str) {
                cmp_value_buff = like_esc::escape(path_str);
                ("LIKE $1 || '%' ESCAPE '$'", cmp_value_buff.as_str())
            } else {
                ("LIKE $1 || '%'", path_str)
            };

            let sql = format!("SELECT path FROM tracks WHERE path {like_query}");
            let paths = sqlx::query_scalar(&sql)
                .bind(cmp_value)
                .fetch_all(&mut **tx)
                .await?;

            Ok(paths)
        }
    }

    /// LibPathStr を曲ファイルのパスとみなすことができるか確認
    ///
    /// 曲のパスとみなすことができるなら `Some(LibTrackPath)` を返す。
    ///
    /// LibTrackPath への変換に失敗した (空文字列だった) 場合は None を返す。
    /// 曲ファイルがそのパスに存在しなかった場合も None を返す。
    async fn path_str_as_track_path<'c>(
        &self,
        tx: &mut PgTransaction,
        path_str: &LibPathStr,
    ) -> Result<Option<LibTrackPath>> {
        match LibTrackPath::from_str(path_str.as_str()) {
            Ok(track_path) => {
                if self.is_exist_path(tx, &track_path).await? {
                    Ok(Some(track_path))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// 指定したパスの曲が存在するか確認
    async fn is_exist_path<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        path: &LibTrackPath,
    ) -> Result<bool> {
        track_sqls::exists_path(tx, path).await
    }

    /// 指定されたフォルダに曲が存在するか確認
    async fn is_exist_in_folder<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        folder_id: i32,
    ) -> Result<bool> {
        let track_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE folder_id = $1"#,
            folder_id,
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(track_count > 0)
    }

    /// 曲のパスを書き換え
    ///
    /// # Arguments
    /// - old_path: 書き換え元の曲のパス
    /// - new_path: 書き換え先の曲のパス
    /// - new_folder_id: 新しい親フォルダのID
    async fn update_path<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        old_path: &LibTrackPath,
        new_path: &LibTrackPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE tracks SET path = $1, folder_id = $2 WHERE path = $3",
            new_path.as_ref() as &str,
            new_folder_id.into_db(),
            old_path.as_ref() as &str,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 曲の再生時間を書き換え
    async fn update_duration<'c>(
        &self,
        tx: &mut PgTransaction<'c>,
        track_id: i32,
        duration: u32,
    ) -> Result<()> {
        let duration_i32: i32 = duration.try_into()?;

        sqlx::query!(
            "UPDATE tracks SET duration = $1 WHERE id = $2",
            duration_i32,
            track_id,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 曲を削除
    ///
    /// # Arguments
    /// - track_id: 削除する曲のID
    async fn delete<'c>(&self, tx: &mut PgTransaction<'c>, track_id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM tracks WHERE id = $1", track_id,)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
