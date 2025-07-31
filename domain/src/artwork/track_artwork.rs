use std::sync::Arc;

use murack_core_media::{
    audio_meta::{AudioPicture, AudioPictureEntry},
    picture::Picture,
};

/// 曲に紐付いたアートワーク1つの情報
#[derive(Debug, PartialEq, Clone)]
pub struct TrackArtwork {
    /// 画像データ
    ///
    /// 同一の画像をキャッシュで共有できるように、Arc で持つ。
    /// (アプリケーション層でやるべきか？)
    pub picture: Arc<Picture>,

    /// 画像タイプ
    ///
    /// FLACやID3で定義された、0〜20の値
    pub picture_type: u8,

    /// 画像の説明
    pub description: String,
}

impl From<AudioPicture> for TrackArtwork {
    fn from(p: AudioPicture) -> Self {
        Self {
            picture: Arc::new(Picture {
                bytes: p.bytes,
                mime_type: p.mime_type,
            }),
            picture_type: p.picture_type,
            description: p.description,
        }
    }
}

impl<'a> From<&'a TrackArtwork> for AudioPictureEntry<'a> {
    fn from(d: &'a TrackArtwork) -> Self {
        Self {
            bytes: &d.picture.bytes,
            mime_type: &d.picture.mime_type,
            picture_type: d.picture_type,
            description: &d.description,
        }
    }
}
