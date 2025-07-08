/// ファイル存在チェックの結果
pub enum ResolveFileExistanceResult {
    /// 解決(次の解決処理へ続行可能)
    Resolved,
    /// 削除することにより解決(このファイルの解決処理の続行は不可能)
    Deleted,
    /// 未解決(このファイルの解決処理の続行は不可能)
    UnResolved,
    /// 解決処理全体の中止
    Terminated,
}
