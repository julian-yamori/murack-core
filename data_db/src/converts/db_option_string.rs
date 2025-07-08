use rusqlite::types::{FromSql, FromSqlResult, ValueRef};

/// DBに登録される、Nullableな文字列値
///
/// DBに登録する際はnull不許可で、空文字列をNone扱いとする。
#[derive(Debug, PartialEq, Clone, Default)]
pub struct DbOptionString(String);

impl DbOptionString {
    /// 空値のインスタンスを作成
    pub fn none() -> Self {
        Self(String::default())
    }

    // 現状ではテストに使ってる
    pub fn as_nonnull_str(&self) -> &str {
        &self.0
    }
}

impl From<Option<String>> for DbOptionString {
    fn from(s: Option<String>) -> Self {
        Self(s.unwrap_or_default())
    }
}

impl From<DbOptionString> for Option<String> {
    fn from(s: DbOptionString) -> Self {
        if s.0.is_empty() {
            None
        } else {
            Some(s.0)
        }
    }
}

impl FromSql for DbOptionString {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Ok(Self(value.as_str()?.to_owned()))
    }
}
