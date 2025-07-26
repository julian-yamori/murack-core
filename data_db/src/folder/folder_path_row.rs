use murack_core_domain::NonEmptyString;

/// folder_pathテーブルのレコード
pub struct FolderPathRow {
    /// PK
    pub id: i32,

    /// パス
    pub path: NonEmptyString,

    /// フォルダ名
    pub name: NonEmptyString,

    /// 親フォルダID
    pub parent_id: Option<i32>,
}
