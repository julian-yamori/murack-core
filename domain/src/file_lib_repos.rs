use crate::{
    path::{LibPathStr, LibTrackPath},
    sync::TrackSync,
};
use anyhow::Result;
use mockall::automock;
use murack_core_media::audio_meta::AudioMetaData;
use std::path::{Path, PathBuf};

/// ファイルシステム上のライブラリを扱うリポジトリ
#[automock]
pub trait FileLibraryRepository {
    /// パス文字列で指定されたパスが存在するか確認
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - path_str: 確認対象のライブラリ内パス
    ///
    /// # Returns
    /// ファイル、もしくはフォルダが存在する場合true、どちらもなければfalse
    fn is_exist_path_str(&self, lib_root: &Path, path_str: &LibPathStr) -> Result<bool>;

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
    fn search_by_lib_path(&self, lib_root: &Path, target: &LibPathStr)
    -> Result<Vec<LibTrackPath>>;

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
    fn search_track_outside_lib(&self, target: &Path) -> Result<Vec<PathBuf>>;

    /// 曲のオーディオメタデータを読み込み
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - track_path: 取得対象の曲のライブラリ内パス
    fn read_audio_meta(&self, lib_root: &Path, track_path: &LibTrackPath) -> Result<AudioMetaData>;

    /// DBと連携する曲データを読み込み
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - track_path: 取得対象の曲のライブラリ内パス
    fn read_track_sync(&self, lib_root: &Path, track_path: &LibTrackPath) -> Result<TrackSync>;

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
    ) -> Result<()>;

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
    ) -> Result<()>;

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
    ) -> Result<()>;

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
    ) -> Result<()>;

    /// パス文字列を指定してライブラリ内のファイル/フォルダを移動
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - src: 移動元のライブラリ内パス
    /// - dest: 移動先のライブラリ内パス
    fn move_path_str(&self, lib_root: &Path, src: &LibPathStr, dest: &LibPathStr) -> Result<()>;

    /// ライブラリから曲を削除
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - target: 削除対象の曲のライブラリ内パス
    fn delete_track(&self, lib_root: &Path, target: &LibTrackPath) -> Result<()>;

    /// パス文字列を指定してライブラリから削除
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - target: 削除対象のライブラリ内パス
    ///
    /// # Errors
    /// - alk_base_2_domain::Error::PathStrNotFound: 指定されたパスが見つからなかった場合
    fn delete_path_str(&self, lib_root: &Path, target: &LibPathStr) -> Result<()>;

    /// ライブラリから曲をゴミ箱に移動
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - target: 削除対象の曲のライブラリ内パス
    fn trash_track(&self, lib_root: &Path, target: &LibTrackPath) -> Result<()>;

    /// パス文字列を指定してライブラリからゴミ箱に移動
    ///
    /// # Arguments
    /// - lib_root: ライブラリルートの絶対パス
    /// - target: 削除対象の曲のライブラリ内パス
    ///
    /// # Errors
    /// - alk_base_2_domain::Error::PathStrNotFound: 指定されたパスが見つからなかった場合
    fn trash_path_str(&self, lib_root: &Path, target: &LibPathStr) -> Result<()>;
}
