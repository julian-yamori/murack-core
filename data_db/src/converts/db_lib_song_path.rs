use domain::path::LibSongPath;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

/// DBに保存されるLibSongPath
#[derive(Debug, PartialEq, Clone)]
pub struct DbLibSongPath(LibSongPath);

impl FromSql for DbLibSongPath {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        Ok(Self(LibSongPath::new(s)))
    }
}

impl ToSql for DbLibSongPath {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        let s = self.0.as_str();
        Ok(ToSqlOutput::Borrowed(ValueRef::from(s)))
    }
}

impl From<LibSongPath> for DbLibSongPath {
    fn from(value: LibSongPath) -> Self {
        Self(value)
    }
}

impl From<DbLibSongPath> for LibSongPath {
    fn from(value: DbLibSongPath) -> Self {
        value.0
    }
}

/// DBに保存されるLibSongPathの参照
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DbLibSongPathRef<'a>(&'a LibSongPath);

impl DbLibSongPathRef<'_> {
    /// 所有権あり版と値が等しいか確認
    pub fn assert_eq_buf(&self, buf: &DbLibSongPath) {
        assert_eq!(self.0, &buf.0)
    }
}

impl ToSql for DbLibSongPathRef<'_> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        let s = self.0.as_str();
        Ok(ToSqlOutput::Borrowed(ValueRef::from(s)))
    }
}

impl<'a> From<&'a LibSongPath> for DbLibSongPathRef<'a> {
    fn from(value: &'a LibSongPath) -> Self {
        Self(value)
    }
}
