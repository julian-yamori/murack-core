use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{NonEmptyString, path::PathError};

/// 文字列表現による、ライブラリ内のパス
///
/// ディレクトリを指すのかファイルを指すのかが不明瞭。
/// ユーザーからのパス指定に使用する。
#[derive(Debug, PartialEq, Clone)]
pub struct LibPathStr(NonEmptyString);

impl LibPathStr {
    /// このパス文字列の絶対パスを取得
    ///
    /// # Arguments
    /// - root: ライブラリルートの絶対パス
    pub fn abs(&self, root: &Path) -> PathBuf {
        let mut buf = root.to_path_buf();
        buf.push(self.0.as_ref() as &str);
        buf
    }
}

impl TryFrom<String> for LibPathStr {
    type Error = PathError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(Self(NonEmptyString::try_from(s)?))
    }
}

impl FromStr for LibPathStr {
    type Err = PathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_string().try_into()
    }
}

impl From<LibPathStr> for NonEmptyString {
    fn from(value: LibPathStr) -> Self {
        value.0
    }
}

impl AsRef<Path> for LibPathStr {
    fn as_ref(&self) -> &Path {
        Path::new(self.0.as_ref() as &str)
    }
}

impl fmt::Display for LibPathStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
