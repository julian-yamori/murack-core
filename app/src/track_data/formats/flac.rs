//! FLACフォーマット取扱

use std::path::Path;

use chrono::NaiveDate;
use metaflac::{
    Tag,
    block::{Block, BlockType, PictureType, StreamInfo, VorbisComment},
};

use crate::track_data::{FileMidMetadata, TrackArtwork};

const KEY_COMPOSER: &str = "COMPOSER";
const KEY_TRACK_NUMBER: &str = "TRACKNUMBER";
const KEY_TRACK_MAX: &str = "TOTALTRACKS";
const KEY_DISC_NUMBER: &str = "DISCNUMBER";
const KEY_DISC_MAX: &str = "TOTALDISCS";
const KEY_DATE: &str = "DATE";
const KEY_MEMO: &str = "DESCRIPTION";

/// ファイルからメタデータを読み込み
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
/// # Returns
/// オーディオファイルのメタデータ
pub fn read(path: &Path) -> Result<FileMidMetadata, FlacError> {
    let tag = Tag::read_from_path(path)?;

    let si = tag
        .get_streaminfo()
        .ok_or(FlacError::StreamInfoBlockNotFound)?;
    let v = tag
        .vorbis_comments()
        .ok_or(FlacError::VorbisCommentBlockNotFound)?;

    Ok(FileMidMetadata {
        duration: get_duration(si),
        title: vorbis_get_str(v.title()),
        artist: vorbis_get_str(v.artist()),
        album: vorbis_get_str(v.album()),
        genre: vorbis_get_str(v.genre()),
        album_artist: vorbis_get_str(v.album_artist()),
        composer: vorbis_get_str(v.get(KEY_COMPOSER)),
        track_number: v.track().map(|x| x as i32),
        track_max: v.total_tracks().map(|x| x as i32),
        disc_number: vorbis_get_str_to_int(v, KEY_DISC_NUMBER)?,
        disc_max: vorbis_get_str_to_int(v, KEY_DISC_MAX)?,
        release_date: get_release_date(v)?,
        memo: vorbis_get_str(v.get(KEY_MEMO)),
        //lyrics: vorbis_get_str(v.lyrics()),
        artworks: get_artworks(&tag),
    })
}

/// ファイルのメタデータを上書き
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
/// - track: 書き込む曲の情報
pub fn overwrite(path: &Path, track: FileMidMetadata) -> Result<(), FlacError> {
    let mut tag = Tag::read_from_path(path)?;
    let v = tag.vorbis_comments_mut();

    v.set_title(str_to_vec(track.title));
    v.set_artist(str_to_vec(track.artist));
    v.set_album(str_to_vec(track.album));
    v.set_genre(str_to_vec(track.genre));
    v.set_album_artist(str_to_vec(track.album_artist));
    v.set(KEY_COMPOSER, str_to_vec(track.composer));

    v.set(KEY_TRACK_NUMBER, int_to_track_number(&track.track_number));
    v.set(KEY_TRACK_MAX, int_to_track_number(&track.track_max));
    v.set(KEY_DISC_NUMBER, int_to_track_number(&track.disc_number));
    v.set(KEY_DISC_MAX, int_to_track_number(&track.disc_max));

    match track.release_date {
        Some(date) => {
            let s = date.format("%Y-%m-%d").to_string();
            v.set(KEY_DATE, vec![s]);
        }
        None => v.remove(KEY_DATE),
    }

    v.set(KEY_MEMO, str_to_vec(track.memo));
    //v.set_lyrics(str_to_vec(&track.lyrics));

    tag.remove_blocks(BlockType::Picture);
    for artwork in track.artworks {
        /* こちらだとdescriptionが書き込めない
        tag.add_picture(
            artwork.mime_type.to_owned(),
            picture_type_from_u8(artwork.picture_type),
            artwork.image.to_owned(),
        );
        */
        let mut picture = metaflac::block::Picture::new();
        picture.mime_type = artwork.mime_type;
        picture.picture_type = picture_type_from_u8(artwork.picture_type);
        picture.description = artwork.description;
        picture.data = artwork.image;
        //TODO サイズ等の情報が書き込まれない。add_pictureでも同様。
        tag.push_block(Block::Picture(picture));
    }

    tag.write_to_path(path)?;

    Ok(())
}

/// streaminfoからdurationを取得
fn get_duration(si: &StreamInfo) -> u32 {
    //桁溢れ回避のため、doubleに直して計算

    let sr = match si.sample_rate {
        0 => 44100.0,
        _ => si.sample_rate as f64,
    };
    let total_samples = si.total_samples as f64;

    (total_samples / sr * 1000.0) as u32
}
/// VorbisCommentから文字列値を取得
fn vorbis_get_str(values: Option<&Vec<String>>) -> Option<String> {
    vorbis_get_str_ref(values).cloned()
}
/// VorbisCommentから文字列値を取得（参照のまま）
fn vorbis_get_str_ref(values: Option<&Vec<String>>) -> Option<&String> {
    values.and_then(|vec| vec.iter().next())
}
/// VorbisCommentの文字列値から整数値を取得
fn vorbis_get_str_to_int(vc: &VorbisComment, key: &str) -> Result<Option<i32>, FlacError> {
    let op = vc.get(key).and_then(|vec| vec.iter().next());
    match op {
        Some(s) => match s.parse::<i32>() {
            Ok(i) => Ok(Some(i)),
            Err(_) => Err(FlacError::FailedToParseInteger {
                key: key.to_owned(),
                value: s.clone(),
            }),
        },
        None => Ok(None),
    }
}
/// VorbisCommentからリリース日を取得
fn get_release_date(vc: &VorbisComment) -> Result<Option<NaiveDate>, FlacError> {
    match vorbis_get_str_ref(vc.get(KEY_DATE)) {
        Some(s) => match NaiveDate::parse_from_str(s.as_ref(), "%Y-%m-%d") {
            Ok(date) => Ok(Some(date)),
            Err(_) => Err(FlacError::FailedToParseDate {
                key: KEY_DATE.to_string(),
                value: s.clone(),
            }),
        },
        None => Ok(None),
    }
}
/// アートワークリストを取得
fn get_artworks(tag: &Tag) -> Vec<TrackArtwork> {
    tag.pictures()
        .map(|tag_pic| TrackArtwork {
            image: tag_pic.data.clone(),
            mime_type: tag_pic.mime_type.clone(),
            picture_type: u8_from_picture_type(tag_pic.picture_type),
            description: tag_pic.description.clone(),
        })
        .collect()
}
/// PictureTypeをenumからu8に変換
fn u8_from_picture_type(t: PictureType) -> u8 {
    match t {
        PictureType::Other => 0,
        PictureType::Icon => 1,
        PictureType::OtherIcon => 2,
        PictureType::CoverFront => 3,
        PictureType::CoverBack => 4,
        PictureType::Leaflet => 5,
        PictureType::Media => 6,
        PictureType::LeadArtist => 7,
        PictureType::Artist => 8,
        PictureType::Conductor => 9,
        PictureType::Band => 10,
        PictureType::Composer => 11,
        PictureType::Lyricist => 12,
        PictureType::RecordingLocation => 13,
        PictureType::DuringRecording => 14,
        PictureType::DuringPerformance => 15,
        PictureType::ScreenCapture => 16,
        PictureType::BrightFish => 17,
        PictureType::Illustration => 18,
        PictureType::BandLogo => 19,
        PictureType::PublisherLogo => 20,
    }
}
/// PictureTypeをu8からenumに変換
fn picture_type_from_u8(i: u8) -> PictureType {
    match i {
        0 => PictureType::Other,
        1 => PictureType::Icon,
        2 => PictureType::OtherIcon,
        3 => PictureType::CoverFront,
        4 => PictureType::CoverBack,
        5 => PictureType::Leaflet,
        6 => PictureType::Media,
        7 => PictureType::LeadArtist,
        8 => PictureType::Artist,
        9 => PictureType::Conductor,
        10 => PictureType::Band,
        11 => PictureType::Composer,
        12 => PictureType::Lyricist,
        13 => PictureType::RecordingLocation,
        14 => PictureType::DuringRecording,
        15 => PictureType::DuringPerformance,
        16 => PictureType::ScreenCapture,
        17 => PictureType::BrightFish,
        18 => PictureType::Illustration,
        19 => PictureType::BandLogo,
        20 => PictureType::PublisherLogo,
        _ => PictureType::CoverFront,
    }
}
/// 文字列値をVecに変換（metadata書き込み用）
fn str_to_vec(s: Option<String>) -> Vec<String> {
    match s {
        Some(s) => vec![s],
        None => vec![],
    }
}
/// 整数値をTrackNumber等書き込み用のVecに変換
fn int_to_track_number(i: &Option<i32>) -> Vec<String> {
    match i {
        Some(n) => vec![n.to_string()],
        None => vec![],
    }
}

/// Flac 曲データ関連のエラー
#[derive(thiserror::Error, Debug)]
pub enum FlacError {
    #[error(transparent)]
    Metafrac(#[from] metaflac::Error),

    #[error("StreamInfoブロックがありません")]
    StreamInfoBlockNotFound,

    #[error("VorbisCommentブロックがありません")]
    VorbisCommentBlockNotFound,

    #[error("VorbisCommentの値を数値に変換できませんでした: {key}={value}")]
    FailedToParseInteger {
        /// 変換に失敗した値のVorbisComment key
        key: String,
        /// 変換しようとした文字列
        value: String,
    },

    #[error("VorbisCommentの値を日付に変換できませんでした: {key}={value}")]
    FailedToParseDate {
        /// 変換に失敗した値のVorbisComment key
        key: String,
        /// 変換しようとした文字列
        value: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vorbis_get_str() {
        assert_eq!(vorbis_get_str(None), None);
        assert_eq!(vorbis_get_str(Some(&vec!())), None);
        assert_eq!(
            vorbis_get_str(Some(&vec!("hoge".to_owned()))),
            Some("hoge".to_owned())
        );
        assert_eq!(
            vorbis_get_str(Some(&vec!("hoge".to_owned(), "fuga".to_owned()))),
            Some("hoge".to_owned())
        );
    }

    #[test]
    fn test_int_to_track_number() {
        assert_eq!(int_to_track_number(&None), Vec::<String>::new());
        assert_eq!(int_to_track_number(&Some(5)), vec!["5"]);
        assert_eq!(int_to_track_number(&Some(12)), vec!["12"]);
    }
}
