use crate::Error;
use domain::playlist::PlaylistType;
use num_traits::FromPrimitive;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

/// DBに保存されるPlaylistType
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DbPlaylistType(PlaylistType);

impl FromSql for DbPlaylistType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let num = value.as_i64()?;
        match FromPrimitive::from_i64(num) {
            Some(en) => Ok(Self(en)),
            None => Err(FromSqlError::Other(Box::new(Error::InvalidPlaylistType {
                type_num: num,
            }))),
        }
    }
}

impl ToSql for DbPlaylistType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.0 as i64)))
    }
}

impl From<PlaylistType> for DbPlaylistType {
    fn from(value: PlaylistType) -> Self {
        Self(value)
    }
}

impl From<DbPlaylistType> for PlaylistType {
    fn from(value: DbPlaylistType) -> Self {
        value.0
    }
}
