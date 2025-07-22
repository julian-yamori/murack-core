use std::path::{Path, PathBuf};

use anyhow::Result;
use murack_core_domain::{
    FileLibraryRepository,
    path::{LibPathStr, LibSongPath},
    sync::SongSync,
};
use murack_core_media::audio_meta::AudioMetaData;

use super::{copy, delete, mod_move, search, song_sync};

/// FileLibraryRepositoryの本実装
pub struct FileLibraryRepositoryImpl {}

impl FileLibraryRepository for FileLibraryRepositoryImpl {
    /// パス文字列で指定されたパスが存在するか確認
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - path_str: 確認対象のライブラリ内パス
    ///
    /// # Returns
    /// ファイル、もしくはフォルダが存在する場合true、どちらもなければfalse
    fn is_exist_path_str(&self, lib_root: &Path, path_str: &LibPathStr) -> Result<bool> {
        Ok(path_str.abs(lib_root).exists())
    }

    /// ライブラリのフォルダ内で、指定パスに該当する曲のパスを列挙
    ///
    /// 指定されたパスがファイルなら、そのファイルパスを返す。
    /// 指定されたパスがディレクトリなら、そのディレクトリに含まれる全音声ファイルのパスを返す。(子ディレクトリ内も含む)
    ///
    /// 指定パスにファイル/ディレクトリがない、
    /// もしくは指定ディレクトリ内に音声ファイルがない場合、
    /// 長さ0のVecを返す。
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - target: 検索対象のライブラリ内パス
    fn search_by_lib_path(&self, lib_root: &Path, target: &LibPathStr) -> Result<Vec<LibSongPath>> {
        search::search_by_lib_path(lib_root, target)
    }

    /// ライブラリ外で、指定パスに該当する曲のパスを列挙
    ///
    /// 指定されたパスがファイルなら、そのファイルパスを返す。
    /// 指定されたパスがディレクトリなら、そのディレクトリに含まれる全音声ファイルのパスを返す。(子ディレクトリ内も含む)
    ///
    /// 指定パスにファイル/ディレクトリがない、
    /// もしくは指定ディレクトリ内に音声ファイルがない場合、
    /// 長さ0のVecを返す。
    ///
    /// # Arguments
    /// - target: 検索する絶対パス
    fn search_song_outside_lib(&self, target: &Path) -> Result<Vec<PathBuf>> {
        search::search_song_outside_lib(target)
    }

    /// 曲のオーディオメタデータを読み込み
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - song_path: 取得対象の曲のライブラリ内パス
    fn read_audio_meta(&self, lib_root: &Path, song_path: &LibSongPath) -> Result<AudioMetaData> {
        song_sync::read_metadata(lib_root, song_path)
    }

    /// DBと連携する曲データを読み込み
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - song_path: 取得対象の曲のライブラリ内パス
    fn read_song_sync(&self, lib_root: &Path, song_path: &LibSongPath) -> Result<SongSync> {
        song_sync::read(lib_root, song_path)
    }

    /// DBと連携する曲データを上書き
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - song_path: 保存対象の曲のライブラリ内パス
    /// - song_sync: 保存する曲データ
    fn overwrite_song_sync(
        &self,
        lib_root: &Path,
        song_path: &LibSongPath,
        song_sync: &SongSync,
    ) -> Result<()> {
        song_sync::overwrite(lib_root, song_path, song_sync)
    }

    /// ライブラリからライブラリへ、曲データをコピー
    ///
    /// コピー先にファイルが既に存在する場合はエラーとする。
    ///
    /// # Arguments
    /// - src_lib: コピー元のライブラリのルート絶対パス
    /// - dest_lib: コピー先のライブラリのルート絶対パス
    /// - target: コピーする曲のライブラリ内パス
    fn copy_song_over_lib(
        &self,
        src_lib: &Path,
        dest_lib: &Path,
        target: &LibSongPath,
    ) -> Result<()> {
        copy::copy_song_over_lib(src_lib, dest_lib, target)
    }

    /// ライブラリ外からライブラリ内にファイルをコピー
    ///
    /// lrcファイルは扱わない
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - song_path: コピー先のライブラリ内パス
    /// - src_path: コピー元のライブラリ外のファイル絶対パス
    fn copy_from_outside_lib(
        &self,
        lib_root: &Path,
        song_path: &LibSongPath,
        src_path: &Path,
    ) -> Result<()> {
        copy::copy_from_outside_lib(lib_root, song_path, src_path)
    }

    /// ライブラリからライブラリへ、曲データを上書き
    ///
    /// # Arguments
    /// - src_lib: コピー元のライブラリのルート絶対パス
    /// - dest_lib: 上書き先のライブラリのルート絶対パス
    /// - target: コピーする曲のライブラリ内パス
    fn overwrite_song_over_lib(
        &self,
        src_lib: &Path,
        dest_lib: &Path,
        target: &LibSongPath,
    ) -> Result<()> {
        copy::overwrite_song_over_lib(src_lib, dest_lib, target)
    }

    /// パス文字列を指定してライブラリ内のファイル/フォルダを移動
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - src: 移動元のライブラリ内パス
    /// - dest: 移動先のライブラリ内パス
    fn move_path_str(&self, lib_root: &Path, src: &LibPathStr, dest: &LibPathStr) -> Result<()> {
        mod_move::move_path_str(lib_root, src, dest)
    }

    /// ライブラリから曲を削除
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - target: 削除対象の曲のライブラリ内パス
    fn delete_song(&self, lib_root: &Path, target: &LibSongPath) -> Result<()> {
        delete::delete_song(lib_root, target)
    }

    /// パス文字列を指定してライブラリから削除
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - target: 削除対象のライブラリ内パス
    fn delete_path_str(&self, lib_root: &Path, target: &LibPathStr) -> Result<()> {
        delete::delete_path_str(lib_root, target)
    }

    /// ライブラリから曲をゴミ箱に移動
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - target: 削除対象の曲のライブラリ内パス
    fn trash_song(&self, lib_root: &Path, target: &LibSongPath) -> Result<()> {
        delete::trash_song(lib_root, target)
    }

    /// パス文字列を指定してライブラリからゴミ箱に移動
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - target: 削除対象の曲のライブラリ内パス
    fn trash_path_str(&self, lib_root: &Path, target: &LibPathStr) -> Result<()> {
        delete::trash_path_str(lib_root, target)
    }
}
