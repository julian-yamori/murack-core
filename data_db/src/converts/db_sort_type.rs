use crate::Error;
use domain::playlist::SortType;
use num_traits::FromPrimitive;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

/// DBに保存されるSortType
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DbSortType(SortType);

impl FromSql for DbSortType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let num = value.as_i64()?;
        match FromPrimitive::from_i64(num) {
            Some(en) => Ok(Self(en)),
            None => Err(FromSqlError::Other(Box::new(Error::InvalidSortType {
                type_num: num,
            }))),
        }
    }
}

impl ToSql for DbSortType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.0 as i64)))
    }
}

impl From<SortType> for DbSortType {
    fn from(value: SortType) -> Self {
        Self(value)
    }
}

impl From<DbSortType> for SortType {
    fn from(value: DbSortType) -> Self {
        value.0
    }
}
