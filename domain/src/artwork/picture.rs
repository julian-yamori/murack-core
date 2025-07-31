use std::{fmt, io::Cursor};

use anyhow::{Context, Result};
use image::{ImageResult, codecs::jpeg::JpegEncoder, imageops::FilterType};

/// 縮小版アートワークのサイズ
const MINI_SIZE: u32 = 112;
/// 縮小版アートワーク変換の画質
const MINI_QUALITY: u8 = 85;

/// 画像データ
#[derive(PartialEq, Clone)]
pub struct Picture {
    /// 画像のバイトデータ
    pub bytes: Vec<u8>,

    /// 画像データのMIMEタイプ
    pub mime_type: String,
}

impl Picture {
    /// 画像のMD5ハッシュ値を取得
    pub fn hash(&self) -> Vec<u8> {
        md5::compute(&self.bytes).0.to_vec()
    }

    /// アートワークの縮小版画像データを作成
    pub fn artwork_mini_image(&self) -> Result<Vec<u8>> {
        make_mini_image(&self.bytes).with_context(|| "failed to make mini artwork".to_owned())
    }
}

fn make_mini_image(bytes: &[u8]) -> ImageResult<Vec<u8>> {
    //画像を縮小
    let img = image::load_from_memory(bytes)?;
    let resized = img.resize(MINI_SIZE, MINI_SIZE, FilterType::Lanczos3);

    let mut output = Cursor::new(Vec::new());
    // let mut output = ImageBuffer::new(resized.width(), resized.height());
    let encoder = JpegEncoder::new_with_quality(&mut output, MINI_QUALITY);

    resized.write_with_encoder(encoder)?;

    Ok(output.into_inner())
}

impl fmt::Debug for Picture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArtworkSync")
            .field("bytes", &"abbr.")
            .field("mime_type", &self.mime_type)
            .finish()
    }
}
