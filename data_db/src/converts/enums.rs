use domain::folder::FolderIdMayRoot;

pub fn db_from_folder_id_may_root(value: FolderIdMayRoot) -> Option<i32> {
    match value {
        FolderIdMayRoot::Folder(i) => Some(i),
        FolderIdMayRoot::Root => None,
    }
}
pub fn db_into_folder_id_may_root(value: Option<i32>) -> FolderIdMayRoot {
    match value {
        Some(i) => FolderIdMayRoot::Folder(i),
        None => FolderIdMayRoot::Root,
    }
}
