use crate::converts::DbOptionString;

/// folder_pathテーブルのレコード
pub struct FolderPathRow {
    /// PK
    pub id: i32,

    /// パス
    pub path: DbOptionString,

    /// フォルダ名
    pub name: DbOptionString,

    /// 親フォルダID
    pub parent_id: Option<i32>,
}
