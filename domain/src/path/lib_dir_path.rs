use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::NonEmptyString;
use crate::path::PathError;

/// ライブラリ内のディレクトリのパス
///
/// 例えば PC 内の場合、`Config.pc_lib` のパスから見てどの位置にディレクトリがあるかを示す。
///
/// また、DBの`folder_path.path`に保存する値。
///
/// この構造体では、`Config.pc_lib` より下のディレクトリのパスを扱う。`Config.pc_lib` 自体 (ルートディレクトリ) 自体のパスは扱わない。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LibDirPath(
    /// `/` で終わる形のパスを保存する。
    /// （後ろに他のパスをすぐ連結できる形）
    NonEmptyString,
);

impl LibDirPath {
    /// パスを文字列で取得
    ///
    /// `/` で終わる形の文字列を返す。
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// ディレクトリの絶対パスを取得
    ///
    /// # Arguments
    /// - root: ライブラリルートの絶対パス
    pub fn abs(&self, root: &Path) -> PathBuf {
        root.join::<&str>(self.0.as_ref())
    }

    /// 末端のディレクトリ名を取得
    pub fn dir_name(&self) -> &str {
        let len = self.0.len();

        // 末尾以外のスラッシュを検索し、末尾のスラッシュを除いて返す
        match self.0[..len - 1].rfind('/') {
            Some(slash) => &self.0[slash + 1..len - 1],
            None => &self.0[..len - 1],
        }
    }

    /// 親のディレクトリパスを取得
    ///
    /// 親ディレクトリがライブラリルートだった場合、None を返す。
    pub fn parent(&self) -> Option<LibDirPath> {
        let len = self.0.len();

        //末尾以外のスラッシュを検索
        match self.0[..len - 1].rfind('/') {
            Some(slash_pos) => {
                let parent_str = &self.0[..slash_pos + 1];
                let non_empty = NonEmptyString::from_str(parent_str).unwrap();
                Some(Self(non_empty))
            }
            None => None,
        }
    }
}

impl TryFrom<PathBuf> for LibDirPath {
    type Error = PathError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let utf_string = match value.into_os_string().into_string() {
            Ok(s) => s,
            Err(osstr) => {
                return Err(PathError::FailedToDecode { from: osstr });
            }
        };

        Self::try_from(utf_string)
    }
}

impl TryFrom<String> for LibDirPath {
    type Error = PathError;

    /// 文字列からパスインスタンスを生成
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let non_empty = NonEmptyString::try_from(value)?;
        Ok(Self::from(non_empty))
    }
}

impl From<NonEmptyString> for LibDirPath {
    fn from(mut value: NonEmptyString) -> Self {
        //終端が `/` でなければ追加
        if !value.ends_with('/') {
            value.push('/');
        }

        Self(value)
    }
}

impl FromStr for LibDirPath {
    type Err = PathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_string().try_into()
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

    fn str_to_path(s: &str) -> LibDirPath {
        LibDirPath::from_str(s).unwrap()
    }

    #[test]
    fn from_empty_str_should_error() {
        let result = LibDirPath::from_str("");
        assert!(matches!(result, Err(PathError::EmptyString(_))));
    }

    #[test]
    fn test_from() {
        assert_eq!(str_to_path("hoge").0.as_ref() as &str, "hoge/");
        assert_eq!(str_to_path("hoge/").0.as_ref() as &str, "hoge/");
        assert_eq!(str_to_path("hoge/fuga").0.as_ref() as &str, "hoge/fuga/");
        assert_eq!(str_to_path("hoge/fuga/").0.as_ref() as &str, "hoge/fuga/");
    }

    #[test]
    fn test_dir_name() {
        assert_eq!(str_to_path("hoge").dir_name(), "hoge");
        assert_eq!(str_to_path("hoge/fuga").dir_name(), "fuga");
        assert_eq!(str_to_path("hoge/fuga/piyo/").dir_name(), "piyo");
    }

    #[test]
    fn test_parent() {
        assert_eq!(str_to_path("hoge").parent(), None);
        assert_eq!(str_to_path("hoge/").parent(), None);
        assert_eq!(str_to_path("hoge/fuga").parent(), Some(str_to_path("hoge")));
        assert_eq!(
            str_to_path("hoge/fuga/").parent(),
            Some(str_to_path("hoge"))
        );
        assert_eq!(
            str_to_path("hoge/fuga/piyo/").parent(),
            Some(str_to_path("hoge/fuga"))
        );
    }

    #[test]
    fn test_abs() {
        let path = LibDirPath::from_str("hoge/fuga/").unwrap();
        let root = PathBuf::from("/home/taro/Musics");
        assert_eq!(
            path.abs(&root),
            PathBuf::from("/home/taro/Musics/hoge/fuga")
        );
    }
}
