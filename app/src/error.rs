/// WalkBase2 app層のエラー
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// コマンドの引数が不正
    #[error("{msg}")]
    InvalidCommandArgument { msg: String },

    /// Config: root要素がテーブルでない
    #[error("Failed to parse config.\nNot table.")]
    ConfigRootIsNotTable,
    /// Config: 必要なキーが見つからなかった
    #[error("Failed to get \"{key}\" in config.")]
    ConfigNotFound { key: String },
    /// Config: 文字列値が入っているべきキーが文字列でなかった
    #[error("\"{key}\" is not string in config.")]
    ConfigNotString { key: String },
}
