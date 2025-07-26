use serde::{Deserialize, Serialize};

/// ライブラリ内のフォルダのID(Rootを指す場合あり)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FolderIdMayRoot {
    /// ライブラリ内に実在するフォルダ
    Folder(i32),
    /// ライブラリルートのフォルダ
    Root,
}

impl FolderIdMayRoot {
    /// SQL 文で使用する値に変換
    ///
    /// `Option::<i32>::from()` よりも簡潔で、sqlx のマクロでの型エラーも起こしにくい形
    pub fn into_db(self) -> Option<i32> {
        self.into()
    }
}

impl From<Option<i32>> for FolderIdMayRoot {
    fn from(value: Option<i32>) -> Self {
        match value {
            Some(v) => Self::Folder(v),
            None => Self::Root,
        }
    }
}

impl From<FolderIdMayRoot> for Option<i32> {
    fn from(value: FolderIdMayRoot) -> Self {
        match value {
            FolderIdMayRoot::Folder(v) => Some(v),
            FolderIdMayRoot::Root => None,
        }
    }
}

impl Serialize for FolderIdMayRoot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Option::<i32>::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FolderIdMayRoot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::<i32>::deserialize(deserializer).map(Self::from)
    }
}
