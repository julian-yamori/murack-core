use anyhow::Result;
use async_trait::async_trait;
use murack_core_domain::{
    db::DbTransaction,
    folder::FolderIdMayRoot,
    path::{LibDirPath, LibPathStr, LibSongPath},
    song::DbSongRepository,
};

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
        song_sqls::select_id_by_path(tx, path).await
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
        //ルートフォルダ指定なら、全曲
        if path.is_root() {
            song_sqls::select_path_all(tx).await
        } else {
            song_sqls::select_path_begins_directory(tx, path).await
        }
    }

    /// ライブラリ内の全ての曲のパスを取得
    async fn get_path_all<'c>(&self, tx: &mut DbTransaction<'c>) -> Result<Vec<LibSongPath>> {
        song_sqls::select_path_all(tx).await
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
        let song_count =
            song_sqls::count_by_folder_id(tx, FolderIdMayRoot::Folder(folder_id)).await?;
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
        song_sqls::update_path_by_path(tx, old_path, new_path, new_folder_id).await
    }

    /// 曲の再生時間を書き換え
    async fn update_duration<'c>(
        &self,
        tx: &mut DbTransaction<'c>,
        song_id: i32,
        duration: u32,
    ) -> Result<()> {
        song_sqls::update_duration_by_id(tx, song_id, duration).await
    }

    /// 曲を削除
    ///
    /// # Arguments
    /// - song_id: 削除する曲のID
    async fn delete<'c>(&self, tx: &mut DbTransaction<'c>, song_id: i32) -> Result<()> {
        song_sqls::delete(tx, song_id).await
    }
}

#[cfg(test)]
mod tests;
