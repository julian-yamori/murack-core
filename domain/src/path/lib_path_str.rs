use super::{LibDirPath, LibSongPath};
use std::{
    fmt,
    path::{Path, PathBuf},
};

/// ライブラリ内の文字列表現による文字列パス
///
/// ディレクトリを指すのかファイルを指すのかが不明瞭。
/// ユーザーからのパス指定に使用する。
#[derive(Debug, PartialEq, Clone)]
pub struct LibPathStr(String);

impl LibPathStr {
    /// パスを文字列で取得
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// ライブラリのルートディレクトリを示すパスを取得
    pub fn root() -> Self {
        Self("".to_owned())
    }

    /// このパスが、ライブラリのルートディレクトリを示しているか確認
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// このパス文字列の絶対パスを取得
    ///
    /// # Arguments
    /// - root: ライブラリルートの絶対パス
    pub fn abs(&self, root: &Path) -> PathBuf {
        let mut buf = root.to_path_buf();
        buf.push(&self.0);
        buf
    }

    /// このパスが曲を示していると解釈し、曲パスインスタンスを取得
    pub fn to_song_path(&self) -> LibSongPath {
        LibSongPath::new(&self.0)
    }
    /// このパスがディレクトリを示していると解釈し、ディレクトリパスインスタンスを取得
    pub fn to_dir_path(&self) -> LibDirPath {
        LibDirPath::new(&self.0)
    }
}

impl From<String> for LibPathStr {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Display for LibPathStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
