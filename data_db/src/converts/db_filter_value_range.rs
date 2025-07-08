use crate::Error;
use domain::filter::FilterValueRange;
use num_traits::FromPrimitive;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

/// DBに保存されるFilterValueRange
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DbFilterValueRange(FilterValueRange);

impl FromSql for DbFilterValueRange {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let num = value.as_i64()?;
        match FromPrimitive::from_i64(num) {
            Some(en) => Ok(Self(en)),
            None => Err(FromSqlError::Other(Box::new(
                Error::InvalidFilterValueRange { type_num: num },
            ))),
        }
    }
}

impl ToSql for DbFilterValueRange {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.0 as i64)))
    }
}

impl From<FilterValueRange> for DbFilterValueRange {
    fn from(value: FilterValueRange) -> Self {
        Self(value)
    }
}

impl From<DbFilterValueRange> for FilterValueRange {
    fn from(value: DbFilterValueRange) -> Self {
        value.0
    }
}
