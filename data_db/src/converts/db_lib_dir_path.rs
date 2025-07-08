use domain::path::LibDirPath;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

/// DBに保存されるLibDirPath
#[derive(Debug, PartialEq, Clone)]
pub struct DbLibDirPath(LibDirPath);

impl FromSql for DbLibDirPath {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        Ok(Self(LibDirPath::new(s)))
    }
}

impl ToSql for DbLibDirPath {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        let s = self.0.as_str();
        Ok(ToSqlOutput::Borrowed(ValueRef::from(s)))
    }
}

impl From<LibDirPath> for DbLibDirPath {
    fn from(value: LibDirPath) -> Self {
        Self(value)
    }
}

impl From<DbLibDirPath> for LibDirPath {
    fn from(value: DbLibDirPath) -> Self {
        value.0
    }
}

/// DBに保存されるLibDirPathの参照
#[derive(Debug, PartialEq)]
pub struct DbLibDirPathRef<'a>(&'a LibDirPath);

impl ToSql for DbLibDirPathRef<'_> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        let s = self.0.as_str();
        Ok(ToSqlOutput::Borrowed(ValueRef::from(s)))
    }
}

impl<'a> From<&'a LibDirPath> for DbLibDirPathRef<'a> {
    fn from(value: &'a LibDirPath) -> Self {
        Self(value)
    }
}
