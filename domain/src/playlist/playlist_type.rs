use num_derive::FromPrimitive;

/// プレイリストの種類
#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive)]
pub enum PlaylistType {
    /// 通常の、ユーザーが直接管理するプレイリスト
    Normal = 1,
    /// フィルタープレイリスト
    Filter = 2,
    /// プレイリストフォルダ
    Folder = 3,
}
