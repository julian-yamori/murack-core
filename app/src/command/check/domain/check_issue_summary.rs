use std::fmt;

/// チェック処理で検出した問題の簡易情報
pub enum CheckIssueSummary {
    /// PCにファイルが存在しない
    PcNotExists,
    /// PCからの曲データの読み込み失敗
    PcReadFailed { e: anyhow::Error },
    /// DBに曲データが存在しない
    DbNotExists,
    /// DAPにファイルが存在しない
    DapNotExists,
    /// PCとDB間で編集可能データが異なる
    PcDbNotEqualsEditable,
    /// PCとDB間で再生時間が異なる
    PcDbNotEqualsDuration,
    /// PCとDB間でアートワークが異なる
    PcDbNotEqualsArtwork,
    /// PCとDAP間でファイル内容が異なる
    PcDapNotEquals,
}

impl fmt::Display for CheckIssueSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PcNotExists => f.write_str("PCにファイルが存在しません。"),
            Self::PcReadFailed { e } => write!(f, "PCからのデータの読み込みに失敗しました: {e}"),
            Self::DbNotExists => f.write_str("DBにデータが存在しません。"),
            Self::DapNotExists => f.write_str("DAPにデータが存在しません。"),
            Self::PcDbNotEqualsEditable => f.write_str("PCとDBでデータが異なります。"),
            Self::PcDbNotEqualsDuration => f.write_str("PCとDBで再生時間が異なります。"),
            Self::PcDbNotEqualsArtwork => f.write_str("PCとDBでアートワークが異なります。"),
            Self::PcDapNotEquals => f.write_str("PCとDAPでファイル内容が異なります。"),
        }
    }
}
