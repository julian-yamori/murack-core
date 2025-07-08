use anyhow::{Context, Result};
use opencv::{
    core::{Size, Vector},
    imgcodecs, imgproc,
    prelude::*,
};
use std::fmt;

/// 縮小版アートワークのサイズ
const MINI_SIZE: f64 = 112.0;
/// 縮小版アートワーク変換の画質
const MINI_QUALITY: i32 = 85;

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
        open_cv_mini_image(&self.bytes).with_context(|| "failed to make mini artwork".to_owned())
    }
}

fn open_cv_mini_image(bytes: &[u8]) -> opencv::Result<Vec<u8>> {
    //Matに変換
    let mat_image_large = imgcodecs::imdecode(&Mat::from_slice::<u8>(bytes)?, -1)?;

    //サイズ計算
    let x_rate = MINI_SIZE / mat_image_large.cols() as f64;
    let y_rate = MINI_SIZE / mat_image_large.rows() as f64;
    let rate_min = f64::min(x_rate, y_rate);

    //縮小の実行
    let mut mat_image_mini = Mat::default();
    imgproc::resize(
        &mat_image_large,
        &mut mat_image_mini,
        Size::new(0, 0),
        rate_min,
        rate_min,
        imgproc::INTER_LINEAR,
    )?;

    //JPEG変換用の引数を生成(品質指定)
    let params = Vector::<i32>::from(vec![imgcodecs::IMWRITE_JPEG_QUALITY, MINI_QUALITY]);

    //JPEGのバイトデータに変換
    let mut vec_jpg = Vector::<u8>::new();
    imgcodecs::imencode(".jpg", &mat_image_mini, &mut vec_jpg, &params)?;

    Ok(Vec::<u8>::from(vec_jpg))
}

impl fmt::Debug for Picture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArtworkSync")
            .field("bytes", &"abbr.")
            .field("mime_type", &self.mime_type)
            .finish()
    }
}
