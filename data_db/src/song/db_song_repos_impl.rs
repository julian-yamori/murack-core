use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{
    db::DbTransaction,
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibPathStr, LibSongPath},
    song::DbSongRepository,
};
use sqlx::{Row, postgres::PgRow};

use crate::{converts::enums::db_from_folder_id_may_root, like_esc};

use super::song_sqls;

/// HasDbSongRepositoryの本実装
#[derive(new)]
pub struct DbSongRepositoryImpl {}

#[async_trait]
impl DbSongRepository for DbSongRepositoryImpl {
    /// パスから曲IDを取得
    async fn get_id_by_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<Option<i32>> {
        let id = sqlx::query_scalar!("SELECT id FROM tracks WHERE path = $1", path.as_str(),)
            .fetch_optional(&mut **tx.get())
            .await?;

        Ok(id)
    }

    /// 文字列でパスを指定して、該当曲のパスリストを取得
    async fn get_path_by_path_str<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibPathStr,
    ) -> Result<Vec<LibSongPath>> {
        //ディレクトリ指定とみなして検索
        let dir_path = path.to_dir_path();
        let mut list = self.get_path_by_directory(tx, &dir_path).await?;

        //ファイル指定とみなしての検索でヒットしたら追加
        let song_path = path.to_song_path();
        if self.is_exist_path(tx, &song_path).await? {
            list.push(song_path);
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
        tx: &mut DbTransaction<'c>,
        path: &LibDirPath,
    ) -> Result<Vec<LibSongPath>> {
        if path.is_root() {
            //ルートフォルダ指定なら、全曲
            let paths = sqlx::query!("SELECT path FROM tracks")
                .map(|row| LibSongPath::new(row.path))
                .fetch_all(&mut **tx.get())
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
            let paths = sqlx::query(&sql)
                .bind(cmp_value)
                .map(|row: PgRow| LibSongPath::new(row.get::<&str, _>(0)))
                .fetch_all(&mut **tx.get())
                .await?;

            Ok(paths)
        }
    }

    /// 指定したパスの曲が存在するか確認
    async fn is_exist_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        path: &LibSongPath,
    ) -> Result<bool> {
        song_sqls::exists_path(tx, path).await
    }

    /// 指定されたフォルダに曲が存在するか確認
    async fn is_exist_in_folder<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        folder_id: i32,
    ) -> Result<bool> {
        let folder_id_value = db_from_folder_id_may_root(FolderIdMayRoot::Folder(folder_id));
        let song_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM tracks WHERE folder_id IS NOT DISTINCT FROM $1"#,
            folder_id_value,
        )
        .fetch_one(&mut **tx.get())
        .await?;

        Ok(song_count > 0)
    }

    /// 曲のパスを書き換え
    ///
    /// # Arguments
    /// - old_path: 書き換え元の曲のパス
    /// - new_path: 書き換え先の曲のパス
    /// - new_folder_id: 新しい親フォルダのID
    async fn update_path<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        old_path: &LibSongPath,
        new_path: &LibSongPath,
        new_folder_id: FolderIdMayRoot,
    ) -> Result<()> {
        let folder_id_value = db_from_folder_id_may_root(new_folder_id);

        sqlx::query!(
            "UPDATE tracks SET path = $1, folder_id = $2 WHERE path = $3",
            new_path.as_str(),
            folder_id_value,
            old_path.as_str(),
        )
        .execute(&mut **tx.get())
        .await?;

        Ok(())
    }

    /// 曲の再生時間を書き換え
    async fn update_duration<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()> {
        let duration_i32: i32 = duration.try_into()?;

        sqlx::query!(
            "UPDATE tracks SET duration = $1 WHERE id = $2",
            duration_i32,
            song_id,
        )
        .execute(&mut **tx.get())
        .await?;

        Ok(())
    }

    /// 曲を削除
    ///
    /// # Arguments
    /// - song_id: 削除する曲のID
    async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, song_id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM tracks WHERE id = $1", song_id,)
            .execute(&mut **tx.get())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
