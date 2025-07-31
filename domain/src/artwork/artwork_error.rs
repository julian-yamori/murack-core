use image::ImageError;

#[derive(thiserror::Error, Debug)]
pub enum ArtworkError {
    #[error("failed to make mini artwork: {}", .0)]
    FailedToBuildMiniArtwork(ImageError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
