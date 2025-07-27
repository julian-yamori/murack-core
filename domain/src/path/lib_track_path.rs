use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use sqlx::encode::IsNull;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};

use crate::NonEmptyString;
use crate::path::{LibDirPath, PathError};

/// ライブラリ内の曲ファイルの位置を示すパス
///
/// 例えばPC内の場合、
/// configの`base_root`のパスから見て
/// どの位置にファイルがあるかを示す。
///
/// また、DBの`track.path`に保存する値。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LibTrackPath(NonEmptyString);

impl LibTrackPath {
    /// ライブラリルートのパスを指定し、LibTrackPath が指す絶対パスを取得
    pub fn abs(&self, root: &Path) -> PathBuf {
        root.join::<&str>(self.0.as_ref())
    }

    /// ファイル名を取得
    pub fn file_name(&self) -> &str {
        match self.0.rfind('/') {
            Some(slash) => &self.0[slash + 1..],
            None => &self.0,
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
    pub fn with_extension(&self, extension: &str) -> LibTrackPath {
        let mut sbuf = match self.0.rfind('.') {
            Some(dot) => self.0[..dot].to_string(),
            None => self.0.clone().into(),
        };
        sbuf.push('.');
        sbuf.push_str(extension);
        LibTrackPath(sbuf.try_into().unwrap())
    }

    /// 親のディレクトリパスを取得
    pub fn parent(&self) -> LibDirPath {
        let parent_str = match self.0.rfind('/') {
            Some(slash) => &self.0[0..slash + 1],
            //(スラッシュがなければ、親はライブラリルート)
            None => "",
        };

        LibDirPath::new(parent_str)
    }
}

impl TryFrom<PathBuf> for LibTrackPath {
    type Error = PathError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        match value.into_os_string().into_string() {
            Ok(s) => Ok(Self::try_from(s)?),
            Err(osstr) => Err(PathError::FailedToDecode { from: osstr }),
        }
    }
}

impl TryFrom<String> for LibTrackPath {
    type Error = PathError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(NonEmptyString::try_from(value)?))
    }
}

impl From<NonEmptyString> for LibTrackPath {
    fn from(value: NonEmptyString) -> Self {
        Self(value)
    }
}

impl From<LibTrackPath> for PathBuf {
    fn from(value: LibTrackPath) -> Self {
        PathBuf::from(String::from(value.0))
    }
}

impl FromStr for LibTrackPath {
    type Err = PathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_string().try_into()
    }
}

impl AsRef<String> for LibTrackPath {
    fn as_ref(&self) -> &String {
        self.0.as_ref()
    }
}

impl AsRef<str> for LibTrackPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<Path> for LibTrackPath {
    fn as_ref(&self) -> &Path {
        Path::new(<Self as AsRef<str>>::as_ref(self))
    }
}

impl AsRef<NonEmptyString> for LibTrackPath {
    fn as_ref(&self) -> &NonEmptyString {
        &self.0
    }
}

impl fmt::Display for LibTrackPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for LibTrackPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LibTrackPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = NonEmptyString::deserialize(deserializer)?;
        Ok(Self::from(s))
    }
}

impl sqlx::Type<Postgres> for LibTrackPath {
    fn type_info() -> PgTypeInfo {
        NonEmptyString::type_info()
    }

    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        NonEmptyString::compatible(ty)
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for LibTrackPath {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = NonEmptyString::decode(value)?;
        Ok(Self::from(s))
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for LibTrackPath {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        self.0.encode_by_ref(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn str_to_path(s: &str) -> LibTrackPath {
        LibTrackPath::from_str(s).unwrap()
    }

    #[test]
    fn test_file_name() {
        assert_eq!(str_to_path("hoge.mp3").file_name(), "hoge.mp3");
        assert_eq!(str_to_path("hoge/fuga.flac").file_name(), "fuga.flac");
        assert_eq!(str_to_path("hoge/fuga/piyo.m4a").file_name(), "piyo.m4a");
    }

    #[test]
    fn test_file_stem() {
        assert_eq!(str_to_path("hoge.mp3").file_stem(), "hoge");
        assert_eq!(str_to_path("hoge/fuga.flac").file_stem(), "fuga");
        assert_eq!(str_to_path("hoge/fuga/piyo.m4a").file_stem(), "piyo");
        assert_eq!(str_to_path("hoge").file_stem(), "hoge");
        assert_eq!(str_to_path("hoge/fuga/piyo").file_stem(), "piyo");
    }

    #[test_case("hoge/fuga.flac", "mp3", "hoge/fuga.mp3" ; "normal")]
    #[test_case("fuga.flac", "mp3", "fuga.mp3" ; "no_parent")]
    #[test_case("fuga", "mp3", "fuga.mp3" ; "no_ext")]
    fn test_with_extension(before: &str, ext: &str, expect: &str) {
        assert_eq!(str_to_path(before).with_extension(ext), str_to_path(expect));
    }

    #[test]
    fn test_parent() {
        assert!(str_to_path("hoge.mp3").parent().is_root());
        assert_eq!(
            str_to_path("hoge/fuga.flac").parent(),
            LibDirPath::new("hoge")
        );
        assert_eq!(
            str_to_path("hoge/fuga/piyo.m4a").parent(),
            LibDirPath::new("hoge/fuga")
        );
    }
}
