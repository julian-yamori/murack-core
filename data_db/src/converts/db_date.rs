use chrono::NaiveDate;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

/// DBに保存される日付値
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DbDate(NaiveDate);

impl FromSql for DbDate {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(d) => Ok(Self(d)),
            Err(e) => Err(FromSqlError::Other(Box::new(e))),
        }
    }
}

impl ToSql for DbDate {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        let s = self.0.format("%Y-%m-%d").to_string();
        Ok(ToSqlOutput::Owned(Value::from(s)))
    }
}

impl From<NaiveDate> for DbDate {
    fn from(value: NaiveDate) -> Self {
        Self(value)
    }
}

impl From<DbDate> for NaiveDate {
    fn from(value: DbDate) -> Self {
        value.0
    }
}
