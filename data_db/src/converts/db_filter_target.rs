use crate::Error;
use domain::filter::FilterTarget;
use num_traits::FromPrimitive;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

/// DBに保存されるFilterTarget
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DbFilterTarget(FilterTarget);

impl FromSql for DbFilterTarget {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let num = value.as_i64()?;
        match FromPrimitive::from_i64(num) {
            Some(en) => Ok(Self(en)),
            None => Err(FromSqlError::Other(Box::new(Error::InvalidFilterTarget {
                type_num: num,
            }))),
        }
    }
}

impl ToSql for DbFilterTarget {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.0 as i64)))
    }
}

impl From<FilterTarget> for DbFilterTarget {
    fn from(value: FilterTarget) -> Self {
        Self(value)
    }
}

impl From<DbFilterTarget> for FilterTarget {
    fn from(value: DbFilterTarget) -> Self {
        value.0
    }
}
