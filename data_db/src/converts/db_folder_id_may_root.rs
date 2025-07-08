use domain::folder::FolderIdMayRoot;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

/// DBに保存されるFolderIdMayRoot
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DbFolderIdMayRoot(FolderIdMayRoot);

impl FromSql for DbFolderIdMayRoot {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Null => Ok(Self(FolderIdMayRoot::Root)),
            ValueRef::Integer(i) => Ok(Self(FolderIdMayRoot::Folder(i as i32))),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for DbFolderIdMayRoot {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        Ok(ToSqlOutput::Owned(match self.0 {
            FolderIdMayRoot::Folder(i) => Value::Integer(i as i64),
            FolderIdMayRoot::Root => Value::Null,
        }))
    }
}

impl From<FolderIdMayRoot> for DbFolderIdMayRoot {
    fn from(value: FolderIdMayRoot) -> Self {
        Self(value)
    }
}

impl From<DbFolderIdMayRoot> for FolderIdMayRoot {
    fn from(value: DbFolderIdMayRoot) -> Self {
        value.0
    }
}
