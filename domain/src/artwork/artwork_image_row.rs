use std::fmt;

/// artworkテーブルレコードの、画像本体の処理に関わるカラム
#[derive(PartialEq)]
pub struct ArtworkImageRow {
    /// アートワークID
    pub id: i32,

    /// 画像データ
    pub image: Vec<u8>,

    /// 画像データのMIMEタイプ
    pub mime_type: String,
}

impl fmt::Debug for ArtworkImageRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArtworkImageRow")
            .field("image", &"abbr.")
            .field("mime_type", &self.mime_type)
            .finish()
    }
}
