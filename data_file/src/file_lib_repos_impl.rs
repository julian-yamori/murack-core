use std::path::{Path, PathBuf};

use anyhow::Result;
use murack_core_domain::{
    FileLibraryRepository,
    path::{LibPathStr, LibTrackPath},
    sync::TrackSync,
};
use murack_core_media::audio_meta::AudioMetaData;

use super::{copy, delete, mod_move, search, track_sync};

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
    fn search_by_lib_path(
        &self,
        lib_root: &Path,
        target: &LibPathStr,
    ) -> Result<Vec<LibTrackPath>> {
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
    fn search_track_outside_lib(&self, target: &Path) -> Result<Vec<PathBuf>> {
        search::search_track_outside_lib(target)
    }

    /// 曲のオーディオメタデータを読み込み
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - track_path: 取得対象の曲のライブラリ内パス
    fn read_audio_meta(&self, lib_root: &Path, track_path: &LibTrackPath) -> Result<AudioMetaData> {
        track_sync::read_metadata(lib_root, track_path)
    }

    /// DBと連携する曲データを読み込み
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - track_path: 取得対象の曲のライブラリ内パス
    fn read_track_sync(&self, lib_root: &Path, track_path: &LibTrackPath) -> Result<TrackSync> {
        track_sync::read(lib_root, track_path)
    }

    /// DBと連携する曲データを上書き
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - track_path: 保存対象の曲のライブラリ内パス
    /// - track_sync: 保存する曲データ
    fn overwrite_track_sync(
        &self,
        lib_root: &Path,
        track_path: &LibTrackPath,
        track_sync: &TrackSync,
    ) -> Result<()> {
        track_sync::overwrite(lib_root, track_path, track_sync)
    }

    /// ライブラリからライブラリへ、曲データをコピー
    ///
    /// コピー先にファイルが既に存在する場合はエラーとする。
    ///
    /// # Arguments
    /// - src_lib: コピー元のライブラリのルート絶対パス
    /// - dest_lib: コピー先のライブラリのルート絶対パス
    /// - target: コピーする曲のライブラリ内パス
    fn copy_track_over_lib(
        &self,
        src_lib: &Path,
        dest_lib: &Path,
        target: &LibTrackPath,
    ) -> Result<()> {
        copy::copy_track_over_lib(src_lib, dest_lib, target)
    }

    /// ライブラリ外からライブラリ内にファイルをコピー
    ///
    /// lrcファイルは扱わない
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - track_path: コピー先のライブラリ内パス
    /// - src_path: コピー元のライブラリ外のファイル絶対パス
    fn copy_from_outside_lib(
        &self,
        lib_root: &Path,
        track_path: &LibTrackPath,
        src_path: &Path,
    ) -> Result<()> {
        copy::copy_from_outside_lib(lib_root, track_path, src_path)
    }

    /// ライブラリからライブラリへ、曲データを上書き
    ///
    /// # Arguments
    /// - src_lib: コピー元のライブラリのルート絶対パス
    /// - dest_lib: 上書き先のライブラリのルート絶対パス
    /// - target: コピーする曲のライブラリ内パス
    fn overwrite_track_over_lib(
        &self,
        src_lib: &Path,
        dest_lib: &Path,
        target: &LibTrackPath,
    ) -> Result<()> {
        copy::overwrite_track_over_lib(src_lib, dest_lib, target)
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
    fn delete_track(&self, lib_root: &Path, target: &LibTrackPath) -> Result<()> {
        delete::delete_track(lib_root, target)
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
    fn trash_track(&self, lib_root: &Path, target: &LibTrackPath) -> Result<()> {
        delete::trash_track(lib_root, target)
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
