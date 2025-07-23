/// murack-core app層のエラー
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// コマンドの引数が不正
    #[error("{msg}")]
    InvalidCommandArgument { msg: String },
}
