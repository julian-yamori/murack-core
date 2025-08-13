use std::fmt;

/// 画像データ
#[derive(PartialEq, Clone)]
pub struct Picture {
    /// 画像のバイトデータ
    pub bytes: Vec<u8>,

    /// 画像データのMIMEタイプ
    pub mime_type: String,
}

impl fmt::Debug for Picture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArtworkSync")
            .field("bytes", &"abbr.")
            .field("mime_type", &self.mime_type)
            .finish()
    }
}
