use crate::converts::{DbFolderIdMayRoot, DbLibDirPath, DbOptionString};

/// folder_pathテーブルのレコード
pub struct FolderPathRow {
    /// PK
    pub rowid: i32,

    /// パス
    pub path: DbLibDirPath,

    /// フォルダ名
    pub name: DbOptionString,

    /// 親フォルダID
    pub parent_id: DbFolderIdMayRoot,
}
