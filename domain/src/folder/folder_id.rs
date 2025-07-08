/// ライブラリ内のフォルダのID(Rootを指す場合あり)
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FolderIdMayRoot {
    /// ライブラリ内に実在するフォルダ
    Folder(i32),
    /// ライブラリルートのフォルダ
    Root,
}
