use super::LibDirPath;
use std::fmt;
use std::path::{Path, PathBuf};

/// ライブラリ内の曲ファイルの位置を示すパス
///
/// 例えばPC内の場合、
/// configの`base_root`のパスから見て
/// どの位置にファイルがあるかを示す。
///
/// また、DBの`song.path`に保存する値。
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct LibSongPath {
    /// パスの内部実装値
    inner: String,
}

impl LibSongPath {
    /// 文字列からパスインスタンスを生成
    pub fn new(s: impl Into<String>) -> Self {
        Self { inner: s.into() }
    }
    /// std::pathからパスインスタンスを生成
    ///
    /// UTF-8に変換出来なかった場合はNoneを返す。
    pub fn from_path(p: &Path) -> Option<Self> {
        Some(Self::new(p.to_str()?))
    }

    /// パスを文字列で取得
    pub fn as_str(&self) -> &str {
        &self.inner
    }
    /// パスをPathBufとして取得
    ///
    /// ライブラリルートからの相対パスを示す。
    pub fn as_path(&self) -> PathBuf {
        PathBuf::from(&self.inner)
    }

    /// オーディオファイルの絶対パスを取得
    ///
    /// # Arguments
    /// - root: ライブラリルートの絶対パス
    pub fn abs(&self, root: &Path) -> PathBuf {
        let mut buf = root.to_path_buf();
        buf.push(&self.inner);
        buf
    }

    /// ファイル名を取得
    pub fn file_name(&self) -> &str {
        match self.inner.rfind('/') {
            Some(slash) => &self.inner[slash + 1..],
            None => &self.inner,
        }
    }
    /// 拡張子を除いたファイル名を取得
    pub fn file_stem(&self) -> &str {
        let name = self.file_name();
        match name.rfind('.') {
            Some(dot) => &name[..dot],
            None => name,
        }
    }
    /// 拡張子を差し替えて取得
    pub fn with_extension(&self, extension: &str) -> LibSongPath {
        let mut sbuf = match self.inner.rfind('.') {
            Some(dot) => self.inner[..dot].to_owned(),
            None => self.inner.clone(),
        };
        sbuf.push('.');
        sbuf.push_str(extension);
        LibSongPath { inner: sbuf }
    }

    /// 親のディレクトリパスを取得
    pub fn parent(&self) -> LibDirPath {
        let parent_str = match self.inner.rfind('/') {
            Some(slash) => &self.inner[0..slash + 1],
            //(スラッシュがなければ、親はライブラリルート)
            None => "",
        };

        LibDirPath::new(parent_str)
    }
}

impl fmt::Display for LibSongPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn sp(s: &str) -> LibSongPath {
        LibSongPath::new(s)
    }
    fn dp(s: &str) -> LibDirPath {
        LibDirPath::new(s)
    }

    #[test]
    fn test_from() {
        assert_eq!(sp("hoge.mp3").inner, "hoge.mp3");
        assert_eq!(sp("hoge/fuga.flac").inner, "hoge/fuga.flac");
    }

    #[test]
    fn test_file_name() {
        assert_eq!(sp("hoge.mp3").file_name(), "hoge.mp3");
        assert_eq!(sp("hoge/fuga.flac").file_name(), "fuga.flac");
        assert_eq!(sp("hoge/fuga/piyo.m4a").file_name(), "piyo.m4a");
    }

    #[test]
    fn test_file_stem() {
        assert_eq!(sp("hoge.mp3").file_stem(), "hoge");
        assert_eq!(sp("hoge/fuga.flac").file_stem(), "fuga");
        assert_eq!(sp("hoge/fuga/piyo.m4a").file_stem(), "piyo");
        assert_eq!(sp("hoge").file_stem(), "hoge");
        assert_eq!(sp("hoge/fuga/piyo").file_stem(), "piyo");
    }

    #[test_case("hoge/fuga.flac", "mp3", "hoge/fuga.mp3" ; "normal")]
    #[test_case("fuga.flac", "mp3", "fuga.mp3" ; "no_parent")]
    #[test_case("fuga", "mp3", "fuga.mp3" ; "no_ext")]
    fn test_with_extension(before: &str, ext: &str, expect: &str) {
        assert_eq!(
            LibSongPath::new(before).with_extension(ext),
            LibSongPath::new(expect)
        );
    }

    #[test]
    fn test_parent() {
        assert!(sp("hoge.mp3").parent().is_root());
        assert_eq!(sp("hoge/fuga.flac").parent(), dp("hoge"));
        assert_eq!(sp("hoge/fuga/piyo.m4a").parent(), dp("hoge/fuga"));
    }
}
