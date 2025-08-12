//! M4Aフォーマット取扱

use std::path::Path;

use chrono::NaiveDate;
use mp4ameta::{ImgFmt, Tag};

use crate::{
    artwork::{Picture as MurackPicture, TrackArtwork},
    audio_metadata::AudioMetaData,
};

/// ファイルからメタデータを読み込み
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
/// # Returns
/// オーディオファイルのメタデータ
pub fn read(path: &Path) -> Result<AudioMetaData, M4AError> {
    let tag = Tag::read_from_path(path)?;

    Ok(AudioMetaData {
        duration: get_duration(&tag),
        title: opt_str_to_owned(tag.title()),
        artist: opt_str_to_owned(tag.artist()),
        album: opt_str_to_owned(tag.album()),
        genre: opt_str_to_owned(tag.genre()),
        album_artist: opt_str_to_owned(tag.album_artist()),
        composer: opt_str_to_owned(tag.composer()),
        track_number: tag.track_number().map(|x| x as i32),
        track_max: tag.total_tracks().map(|x| x as i32),
        disc_number: tag.disc_number().map(|x| x as i32),
        disc_max: tag.total_discs().map(|x| x as i32),
        release_date: get_release_date(&tag)?,
        memo: opt_str_to_owned(tag.comment()),
        //lyrics: tag.lyrics().map(|s| s.replace("\r", "\n")),
        artworks: get_artworks(&tag),
    })
}

/// ファイルのメタデータを上書き
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
/// - track: 書き込む曲の情報
pub fn overwrite(path: &Path, track: AudioMetaData) -> Result<(), M4AError> {
    let mut tag = Tag::read_from_path(path)?;

    match track.title {
        Some(v) => tag.set_title(v),
        None => tag.remove_title(),
    }
    match track.artist {
        Some(v) => tag.set_artist(v),
        None => tag.remove_artists(),
    }
    match track.album {
        Some(v) => tag.set_album(v),
        None => tag.remove_album(),
    }
    match track.genre {
        Some(v) => tag.set_genre(v),
        None => tag.remove_genres(),
    }
    match track.album_artist {
        Some(v) => tag.set_album_artist(v),
        None => tag.remove_album_artists(),
    }
    match track.composer {
        Some(v) => tag.set_composer(v),
        None => tag.remove_composers(),
    }
    match track.track_number {
        Some(v) => {
            if v == 0 {
                return Err(M4AError::M4ANumberZero {
                    field: "track number".to_owned(),
                });
            }
            tag.set_track_number(v as u16)
        }
        None => tag.remove_track_number(),
    }
    match track.track_max {
        Some(v) => {
            if v == 0 {
                return Err(M4AError::M4ANumberZero {
                    field: "track max".to_owned(),
                });
            }
            tag.set_total_tracks(v as u16)
        }
        None => tag.remove_total_tracks(),
    }
    match track.disc_number {
        Some(v) => {
            if v == 0 {
                return Err(M4AError::M4ANumberZero {
                    field: "disc number".to_owned(),
                });
            }
            tag.set_disc_number(v as u16)
        }
        None => tag.remove_disc_number(),
    }
    match track.disc_max {
        Some(v) => {
            if v == 0 {
                return Err(M4AError::M4ANumberZero {
                    field: "disc max".to_owned(),
                });
            }
            tag.set_total_discs(v as u16)
        }
        None => tag.remove_total_discs(),
    }

    match track.release_date {
        Some(d) => {
            tag.set_year(d.format("%Y-%m-%d").to_string());
        }
        None => {
            tag.remove_year();
        }
    }

    match track.memo {
        Some(v) => tag.set_comment(v),
        None => tag.remove_comments(),
    }

    /*
    match &track.lyrics {
        Some(v) => tag.set_lyrics(v),
        None => tag.remove_lyrics(),
    }
    */

    tag.set_artworks(
        track
            .artworks
            .into_iter()
            .map(|art| {
                Ok(mp4ameta::Img {
                    fmt: match art.picture.mime_type.as_str() {
                        "image/jpeg" => ImgFmt::Jpeg,
                        "image/bmp" => ImgFmt::Bmp,
                        "image/png" => ImgFmt::Png,
                        _ => {
                            return Err(M4AError::UnsupportedArtworkFormat {
                                mime_type: art.picture.mime_type,
                            });
                        }
                    },
                    data: art.picture.bytes,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
    );

    tag.write_to_path(path)?;

    Ok(())
}

/// Tagから再生時間を取得
fn get_duration(tag: &Tag) -> u32 {
    tag.duration().as_millis() as u32
}

/// Option<&str> → Option<String>
fn opt_str_to_owned(o: Option<&str>) -> Option<String> {
    o.map(|s| s.to_owned())
}

/// Tagからリリース日を取得
fn get_release_date(tag: &Tag) -> Result<Option<NaiveDate>, M4AError> {
    match tag.year() {
        Some(s) => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(date) => Ok(Some(date)),
            Err(_) => Err(M4AError::FailedToParseDate {
                value: s.to_owned(),
            }),
        },
        None => Ok(None),
    }
}

/// Tagからアートワークを取得
fn get_artworks(tag: &Tag) -> Vec<TrackArtwork> {
    tag.artworks()
        .map(|img| TrackArtwork {
            picture: MurackPicture {
                bytes: img.data.to_vec(),
                mime_type: match img.fmt {
                    ImgFmt::Bmp => "image/bmp".to_owned(),
                    ImgFmt::Jpeg => "image/jpeg".to_owned(),
                    ImgFmt::Png => "image/png".to_owned(),
                },
            },
            picture_type: 3, //とりあえずCoverFrontとして扱う
            description: String::new(),
        })
        .collect()
}

/// M4A 曲データ関連のエラー
#[derive(thiserror::Error, Debug)]
pub enum M4AError {
    #[error(transparent)]
    Mp4ameta(#[from] mp4ameta::Error),

    #[error("m4aでは{field}に0を設定できません。")]
    M4ANumberZero {
        /// 0を設定しようとした項目名
        field: String,
    },

    #[error("非対応のアートワーク形式です: {mime_type}")]
    UnsupportedArtworkFormat { mime_type: String },

    #[error("値を日付に変換できませんでした: {value}")]
    FailedToParseDate {
        /// 変換しようとした文字列
        value: String,
    },
}
