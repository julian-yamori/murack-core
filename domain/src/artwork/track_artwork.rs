use crate::artwork::Picture;

/// 曲へ紐付けるアートワークの情報
#[derive(Debug, PartialEq, Clone)]
pub struct TrackArtwork {
    pub picture: Picture,

    /// 画像タイプ
    ///
    /// FLACやID3で定義された、0〜20の値
    pub picture_type: u8,

    /// 画像の説明
    pub description: String,
}

/// 曲へ紐付けるアートワークの、登録用データ
#[derive(Debug, PartialEq)]
pub struct TrackArtworkEntry<'a> {
    /// 画像データ
    pub bytes: &'a [u8],

    /// 画像データのMIMEタイプ
    pub mime_type: &'a str,

    /// 画像タイプ
    ///
    /// FLACやID3で定義された、0〜20の値
    pub picture_type: u8,

    /// 画像の説明
    pub description: &'a str,
}

impl<'a> From<&'a TrackArtwork> for TrackArtworkEntry<'a> {
    fn from(d: &'a TrackArtwork) -> Self {
        Self {
            bytes: &d.picture.bytes,
            mime_type: &d.picture.mime_type,
            picture_type: d.picture_type,
            description: &d.description,
        }
    }
}
