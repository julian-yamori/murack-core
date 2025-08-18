use std::num::TryFromIntError;

/// 曲関連のエラー
#[derive(thiserror::Error, Debug)]
pub enum TrackError {
    #[error("duration overflow: {}", .0)]
    DurationOverflow(TryFromIntError),
}
