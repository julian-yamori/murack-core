//! M4Aフォーマット取扱

use super::super::{AudioMetaData, AudioMetaDataEntry, AudioPicture, AudioPictureEntry};
use crate::Error;
use anyhow::Result;
use chrono::NaiveDate;
use mp4ameta::{ImgFmt, Tag};
use std::path::Path;

/// ファイルからメタデータを読み込み
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
/// # Returns
/// オーディオファイルのメタデータ
pub fn read(path: &Path) -> Result<AudioMetaData> {
    let tag = Tag::read_from_path(path)?;

    Ok(AudioMetaData {
        duration: get_duration(&tag)?,
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
/// - song: 書き込む曲の情報
pub fn overwrite(
    path: &Path,
    song: &AudioMetaDataEntry,
    artworks: &[AudioPictureEntry],
) -> Result<()> {
    let mut tag = Tag::read_from_path(path)?;

    match song.title {
        Some(v) => tag.set_title(v),
        None => tag.remove_title(),
    }
    match song.artist {
        Some(v) => tag.set_artist(v),
        None => tag.remove_artists(),
    }
    match song.album {
        Some(v) => tag.set_album(v),
        None => tag.remove_album(),
    }
    match song.genre {
        Some(v) => tag.set_genre(v),
        None => tag.remove_genres(),
    }
    match song.album_artist {
        Some(v) => tag.set_album_artist(v),
        None => tag.remove_album_artists(),
    }
    match song.composer {
        Some(v) => tag.set_composer(v),
        None => tag.remove_composers(),
    }
    match song.track_number {
        Some(v) => {
            if v == 0 {
                return Err(Error::M4ANumberZero {
                    field: "track number".to_owned(),
                }
                .into());
            }
            tag.set_track_number(v as u16)
        }
        None => tag.remove_track_number(),
    }
    match song.track_max {
        Some(v) => {
            if v == 0 {
                return Err(Error::M4ANumberZero {
                    field: "track max".to_owned(),
                }
                .into());
            }
            tag.set_total_tracks(v as u16)
        }
        None => tag.remove_total_tracks(),
    }
    match song.disc_number {
        Some(v) => {
            if v == 0 {
                return Err(Error::M4ANumberZero {
                    field: "disc number".to_owned(),
                }
                .into());
            }
            tag.set_disc_number(v as u16)
        }
        None => tag.remove_disc_number(),
    }
    match song.disc_max {
        Some(v) => {
            if v == 0 {
                return Err(Error::M4ANumberZero {
                    field: "disc max".to_owned(),
                }
                .into());
            }
            tag.set_total_discs(v as u16)
        }
        None => tag.remove_total_discs(),
    }

    match song.release_date {
        Some(d) => {
            tag.set_year(d.format("%Y-%m-%d").to_string());
        }
        None => {
            tag.remove_year();
        }
    }

    match song.memo {
        Some(v) => tag.set_comment(v),
        None => tag.remove_comments(),
    }

    /*
    match &song.lyrics {
        Some(v) => tag.set_lyrics(v),
        None => tag.remove_lyrics(),
    }
    */

    tag.set_artworks(
        artworks
            .iter()
            .map(|art| {
                Ok(mp4ameta::Img {
                    fmt: match art.mime_type {
                        "image/jpeg" => ImgFmt::Jpeg,
                        "image/bmp" => ImgFmt::Bmp,
                        "image/png" => ImgFmt::Png,
                        s => return Err(Error::UnsupportedArtworkFmt { fmt: s.to_owned() }.into()),
                    },
                    data: art.bytes.to_vec(),
                })
            })
            .collect::<Result<Vec<_>>>()?,
    );

    tag.write_to_path(path)?;

    Ok(())
}

/// Tagから再生時間を取得
fn get_duration(tag: &Tag) -> Result<u32> {
    match tag.duration() {
        Some(d) => Ok(d.as_millis() as u32),
        None => Err(Error::InvalidDuration {
            msg: "Duration is None".to_owned(),
        }
        .into()),
    }
}

/// Option<&str> → Option<String>
fn opt_str_to_owned(o: Option<&str>) -> Option<String> {
    o.map(|s| s.to_owned())
}

/// Tagからリリース日を取得
fn get_release_date(tag: &Tag) -> Result<Option<NaiveDate>> {
    match tag.year() {
        Some(s) => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(date) => Ok(Some(date)),
            Err(_) => Err(Error::InvalidReleaseDate {
                value_info: s.to_owned(),
            }
            .into()),
        },
        None => Ok(None),
    }
}

/// Tagからアートワークを取得
fn get_artworks(tag: &Tag) -> Vec<AudioPicture> {
    tag.artworks()
        .map(|img| AudioPicture {
            bytes: img.data.to_vec(),
            mime_type: match img.fmt {
                ImgFmt::Bmp => "image/bmp".to_owned(),
                ImgFmt::Jpeg => "image/jpeg".to_owned(),
                ImgFmt::Png => "image/png".to_owned(),
            },
            picture_type: 3, //とりあえずCoverFrontとして扱う
            description: String::new(),
        })
        .collect()
}
