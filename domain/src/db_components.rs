/// data層DB機能のDIを解決するオブジェクト
pub struct DbComponents {}

impl DbComponents {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DbComponents {
    fn default() -> Self {
        Self::new()
    }
}
