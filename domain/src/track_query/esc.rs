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
