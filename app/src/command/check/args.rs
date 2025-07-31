use murack_core_domain::NonEmptyString;

/// checkコマンドの引数
#[derive(Debug, PartialEq, Clone)]
pub struct CommandCheckArgs {
    /// 確認対象のパス
    ///
    /// None の場合はライブラリ全体をチェックする。
    pub path: Option<NonEmptyString>,

    /// DAPのファイル内容を無視するか
    ///
    /// trueなら、PC間とDAP間でファイル内容を比較しない
    /// (一致として扱う)
    pub ignore_dap_content: bool,
}
