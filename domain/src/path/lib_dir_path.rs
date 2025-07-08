use std::fmt;
use std::path::{Path, PathBuf};

/// ライブラリ内のディレクトリのパス
///
/// 例えばPC内の場合、
/// configの`base_root`のパスから見て
/// どの位置にディレクトリがあるかを示す。
///
/// また、DBの`folder_path.path`に保存する値。
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct LibDirPath {
    /// パスの内部実装値
    ///
    /// ルート以外は`/`で終わる形で保存する。
    /// （後ろに他のパスをすぐ連結できる形）
    inner: String,
}

impl LibDirPath {
    /// 文字列からパスインスタンスを生成
    pub fn new(s: impl Into<String>) -> Self {
        let mut new = Self { inner: s.into() };

        //root以外で終端が`/`でなければ追加
        let i = &mut new.inner;
        if !i.is_empty() && !i.ends_with('/') {
            i.push('/');
        }

        new
    }
    /// std::pathからパスインスタンスを生成
    ///
    /// UTF-8に変換出来なかった場合はNoneを返す。
    pub fn from_path(p: &Path) -> Option<Self> {
        Some(Self::new(p.to_str()?))
    }

    /// ライブラリのルートディレクトリを示すパスを取得
    pub fn root() -> Self {
        Self {
            inner: "".to_owned(),
        }
    }

    /// このパスが、ライブラリのルートディレクトリを示しているか確認
    pub fn is_root(&self) -> bool {
        self.inner.is_empty()
    }

    /// パスを文字列で取得
    ///
    /// ルートの場合は空文字列、
    /// それ以外の場合は`/`で終わる形の文字列を返す。
    pub fn as_str(&self) -> &str {
        &self.inner
    }
    /// パスをPathBufとして取得
    ///
    /// ライブラリルートからの相対パスを示す。
    pub fn as_path(&self) -> PathBuf {
        PathBuf::from(&self.inner)
    }

    /// ディレクトリの絶対パスを取得
    ///
    /// # Arguments
    /// - root: ライブラリルートの絶対パス
    pub fn abs(&self, root: &Path) -> PathBuf {
        root.join(&self.inner)
    }

    /// ディレクトリ名を取得
    ///
    /// # Return
    /// このパスがライブラリルートだった場合、Noneを返す。
    pub fn dir_name(&self) -> Option<&str> {
        let s = &self.inner;
        if s.is_empty() {
            return None;
        }

        //末尾以外のスラッシュを検索
        match s[..s.len() - 1].rfind('/') {
            Some(slash) => Some(&s[slash + 1..s.len() - 1]),
            None => Some(&s[..s.len() - 1]),
        }
    }

    /// 親のディレクトリパスを取得
    ///
    /// # Return
    /// このパスがライブラリルートだった場合、Noneを返す。
    pub fn parent(&self) -> Option<LibDirPath> {
        let s = &self.inner;
        if s.is_empty() {
            return None;
        }

        //末尾以外のスラッシュを検索
        let parent_str = match s[..s.len() - 1].rfind('/') {
            Some(slash) => &s[..slash + 1],
            None => "",
        };

        Some(LibDirPath {
            inner: parent_str.to_owned(),
        })
    }
}

impl fmt::Display for LibDirPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dp(s: &str) -> LibDirPath {
        LibDirPath::new(s)
    }

    #[test]
    fn test_from() {
        assert_eq!(dp("").inner, "");
        assert_eq!(dp("hoge").inner, "hoge/");
        assert_eq!(dp("hoge/").inner, "hoge/");
        assert_eq!(dp("hoge/fuga").inner, "hoge/fuga/");
        assert_eq!(dp("hoge/fuga/").inner, "hoge/fuga/");
    }

    #[test]
    fn test_dir_name() {
        assert_eq!(dp("").dir_name(), None);
        assert_eq!(dp("hoge").dir_name(), Some("hoge"));
        assert_eq!(dp("hoge/fuga").dir_name(), Some("fuga"));
        assert_eq!(dp("hoge/fuga/piyo/").dir_name(), Some("piyo"));
    }

    #[test]
    fn test_parent() {
        assert_eq!(dp("").parent(), None);
        assert!(dp("hoge").parent().unwrap().is_root());
        assert!(dp("hoge/").parent().unwrap().is_root());
        assert_eq!(dp("hoge/fuga").parent(), Some(dp("hoge")));
        assert_eq!(dp("hoge/fuga/").parent(), Some(dp("hoge")));
        assert_eq!(dp("hoge/fuga/piyo/").parent(), Some(dp("hoge/fuga")));
    }

    #[test]
    fn test_abs() {
        let path = LibDirPath::new("hoge/fuga/");
        let root = PathBuf::from("/home/taro/Musics");
        assert_eq!(
            path.abs(&root),
            PathBuf::from("/home/taro/Musics/hoge/fuga")
        );
    }
}
