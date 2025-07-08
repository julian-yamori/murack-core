//! エスケープ関数群
//!
//! # todo
//! 暫定で用意

pub fn esci(v: Option<i32>) -> String {
    match v {
        Some(v) => v.to_string(),
        None => "null".to_owned(),
    }
}
pub fn escs(v: &str) -> String {
    //'で囲む 更にSQLインジェクション等対策
    format!("'{}'", v.replace('\'', "''"))
}
