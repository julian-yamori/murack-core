use std::{ops::Deref, time::Duration};

use sqlx::{
    Postgres,
    encode::IsNull,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef},
};

use crate::track::TrackError;

/// 曲の再生時間
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct TrackDuration(Duration);

impl TrackDuration {
    pub fn as_i32_millis(&self) -> Result<i32, TrackError> {
        self.0
            .as_millis()
            .try_into()
            .map_err(TrackError::DurationOverflow)
    }

    pub fn from_i32_millis(millis: i32) -> Self {
        Self(std::time::Duration::from_millis(millis as u64))
    }
}

impl From<Duration> for TrackDuration {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

impl From<TrackDuration> for Duration {
    fn from(value: TrackDuration) -> Self {
        value.0
    }
}

impl Deref for TrackDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// DB には INTEGER (i32) で保存する

impl sqlx::Type<Postgres> for TrackDuration {
    fn type_info() -> PgTypeInfo {
        <i32 as sqlx::Type<Postgres>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for TrackDuration {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let i = <i32 as sqlx::Decode<Postgres>>::decode(value)?;
        Ok(Self::from_i32_millis(i))
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for TrackDuration {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let i = self.as_i32_millis()?;
        <i32 as sqlx::Encode<Postgres>>::encode_by_ref(&i, buf)
    }
}
