use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;

/// プレイリストの種類
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive, Serialize, Deserialize, Type)]
#[sqlx(type_name = "playlist_type", rename_all = "lowercase")]
pub enum PlaylistType {
    /// 通常の、ユーザーが直接管理するプレイリスト
    Normal = 1,
    /// フィルタープレイリスト
    Filter = 2,
    /// プレイリストフォルダ
    Folder = 3,
}
