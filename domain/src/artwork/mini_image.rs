use std::{fmt, io::Cursor};

use image::{ImageResult, codecs::jpeg::JpegEncoder, imageops::FilterType};

use crate::artwork::ArtworkError;

/// 縮小版アートワークのサイズ
const MINI_SIZE: u32 = 112;
/// 縮小版アートワーク変換の画質
const MINI_QUALITY: u8 = 85;

/// アートワークの、リスト表示などに使用する縮小版画像データ
#[derive(Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(
    feature = "openapi",
    derive(utoipa::ToResponse, utoipa::ToSchema),
    response(content_type = MiniImage::MIME_TYPE),
    schema(format = "binary")
)]
pub struct MiniImage(Vec<u8>);

impl MiniImage {
    pub const MIME_TYPE: &str = "image/jpeg";

    /// 原寸サイズ画像から作成
    pub fn from_original_image(original_image: &[u8]) -> Result<Self, ArtworkError> {
        let image =
            make_mini_image(original_image).map_err(ArtworkError::FailedToBuildMiniArtwork)?;
        Ok(Self(image))
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

impl fmt::Debug for MiniImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MiniImage")
            .field("bytes length", &self.0.len())
            .finish()
    }
}

impl From<Vec<u8>> for MiniImage {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<MiniImage> for Vec<u8> {
    fn from(value: MiniImage) -> Self {
        value.0
    }
}

impl AsRef<[u8]> for MiniImage {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
