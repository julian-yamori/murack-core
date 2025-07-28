use crate::playlist::Playlist;

#[derive(Debug, PartialEq)]
pub struct PlaylistTree {
    pub playlist: Playlist,
    pub children: Vec<PlaylistTree>,
}
