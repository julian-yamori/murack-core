use crate::{NonEmptyString, playlist::Playlist};

#[derive(Debug, PartialEq)]
pub struct PlaylistTree {
    pub playlist: Playlist,
    pub children: Vec<PlaylistTree>,

    /// 親プレイリストの名前リスト
    ///
    /// TODO たぶんこの構造体で持つべきじゃない
    pub parent_names: Vec<NonEmptyString>,
}
