/// 半濁音文字の変換定義
pub struct DakuCnv {
    /// 濁音でなかった場合、もしくは未定義の場合の変換後文字
    pub nml: char,

    /// 濁音の場合の変換後文字
    pub daku: char,

    /// 半濁音の場合の変換後文字
    pub handaku: Option<char>,
}
