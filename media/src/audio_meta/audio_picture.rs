/// オーディオファイルに埋め込まれた画像データのうち、Murack が利用する部分
#[derive(Debug, PartialEq, Clone)]
pub struct AudioPicture {
    /// 画像データ
    pub bytes: Vec<u8>,

    /// 画像データのMIMEタイプ
    pub mime_type: String,

    /// 画像タイプ
    ///
    /// FLACやID3で定義された、0〜20の値
    pub picture_type: u8,

    /// 画像の説明
    pub description: String,
}

/// オーディオファイル画像データの登録用データ
#[derive(Debug, PartialEq)]
pub struct AudioPictureEntry<'a> {
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

impl<'a> From<&'a AudioPicture> for AudioPictureEntry<'a> {
    fn from(d: &'a AudioPicture) -> Self {
        Self {
            bytes: &d.bytes,
            mime_type: &d.mime_type,
            picture_type: d.picture_type,
            description: &d.description,
        }
    }
}
