/// 曲へ紐付けるアートワークの情報
#[derive(Debug, PartialEq, Clone)]
pub struct TrackArtwork {
    /// 画像のバイトデータ
    pub image: Vec<u8>,

    /// 画像データのMIMEタイプ
    pub mime_type: String,

    /// 画像タイプ
    ///
    /// FLACやID3で定義された、0〜20の値
    pub picture_type: u8,

    /// 画像の説明
    pub description: String,
}
