use crate::playlist::playlist_error::PlaylistError;

#[derive(thiserror::Error, Debug)]
pub enum TrackQueryError {
    #[error(transparent)]
    Playlist(#[from] PlaylistError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
