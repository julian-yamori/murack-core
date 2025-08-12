//! MP3フォーマット取扱

use std::{
    borrow::Cow,
    fs::File,
    io::{BufReader, Seek},
    path::{Path, PathBuf},
};

use chrono::{Datelike, NaiveDate};
use id3::{Tag, TagLike};
use mp3_duration::MP3DurationError;

use crate::{
    artwork::{Picture as MurackPicture, TrackArtwork},
    audio_metadata::AudioMetaData,
};

const KEY_COMPOSER: &str = "TCOM";
const KEY_DATE: &str = "TDAT";

/// ファイルからメタデータを読み込み
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
/// # Returns
/// オーディオファイルのメタデータ
pub fn read(path: &Path) -> Result<AudioMetaData, MP3Error> {
    let file = File::open(path).map_err(|e| MP3Error::FileIoError(path.to_owned(), e))?;
    let mut reader = BufReader::new(file);

    let tag = Tag::read_from2(&mut reader)?;

    let duration = read_duration(&mut reader, path)?;

    Ok(AudioMetaData {
        duration,
        title: opt_str_to_owned(tag.title()),
        artist: opt_str_to_owned(tag.artist()),
        album: opt_str_to_owned(tag.album()),
        genre: opt_str_to_owned(tag.genre()),
        album_artist: opt_str_to_owned(tag.album_artist()),
        composer: id3_get_str(&tag, KEY_COMPOSER),
        track_number: tag.track().map(|x| x as i32),
        track_max: tag.total_tracks().map(|x| x as i32),
        disc_number: tag.disc().map(|x| x as i32),
        disc_max: tag.total_discs().map(|x| x as i32),
        release_date: id3_get_release_date(&tag)?,
        memo: id3_get_memo(&tag),
        //lyrics: id3_get_lyrics(&tag),
        artworks: tag
            .pictures()
            .map(|picture| TrackArtwork {
                picture: MurackPicture {
                    bytes: picture.data.clone(),
                    mime_type: picture.mime_type.clone(),
                },
                picture_type: picture.picture_type.into(),
                description: picture.description.clone(),
            })
            .collect(),
    })
}

/// ファイルのメタデータを上書き
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
/// - track: 書き込む曲の情報
pub fn overwrite(path: &Path, track: AudioMetaData) -> Result<(), MP3Error> {
    let mut tag = Tag::read_from_path(path)?;

    match track.title {
        Some(v) => tag.set_title(v),
        None => tag.remove_title(),
    }
    match track.artist {
        Some(v) => tag.set_artist(v),
        None => tag.remove_artist(),
    }
    match track.album {
        Some(v) => tag.set_album(v),
        None => tag.remove_album(),
    }
    match track.genre {
        Some(v) => tag.set_genre(v),
        None => tag.remove_genre(),
    }
    match track.album_artist {
        Some(v) => tag.set_album_artist(v),
        None => tag.remove_album_artist(),
    }
    match track.composer {
        Some(v) => tag.set_text(KEY_COMPOSER, v),
        None => {
            tag.remove(KEY_COMPOSER);
        }
    }
    match track.track_number {
        Some(v) => tag.set_track(v as u32),
        None => tag.remove_track(),
    }
    match track.track_max {
        Some(v) => tag.set_total_tracks(v as u32),
        None => tag.remove_total_tracks(),
    }
    match track.disc_number {
        Some(v) => tag.set_disc(v as u32),
        None => tag.remove_disc(),
    }
    match track.disc_max {
        Some(v) => tag.set_total_discs(v as u32),
        None => tag.remove_total_discs(),
    }

    id3_set_release_date(&mut tag, &track.release_date);

    tag.remove_comment(Some(""), None);
    if let Some(s) = track.memo {
        tag.add_frame(id3::frame::Comment {
            lang: "".to_owned(),
            description: "".to_owned(),
            text: s.to_owned(),
        });
    }
    /*
    tag.remove_all_lyrics();
    if let Some(s) = &track.lyrics {
        tag.add_lyrics(id3::frame::Lyrics {
            lang: "".to_owned(),
            description: "".to_owned(),
            text: s.to_owned(),
        });
    }
    */

    id3_set_artworks(&mut tag, track.artworks)?;

    tag.write_to_path(path, id3::Version::Id3v23)?;
    Ok(())
}

/// MP3から再生時間を読み込み
///
/// # Arguments
/// - reader: 読み込み元のファイルリーダー
/// - path: 読み込むファイルのパス（エラー情報用）
fn read_duration(reader: &mut BufReader<File>, path: &Path) -> Result<u32, MP3Error> {
    let offset = reader
        .stream_position()
        .map_err(|e| MP3Error::FileIoError(path.to_owned(), e))?;

    match mp3_duration::from_read(reader) {
        Ok(d) => Ok(d.as_millis() as u32),
        Err(mut e) => {
            e.offset += offset as usize;
            Err(MP3Error::DurationError(e))
        }
    }
}

/// Option<&str> → Option<String>
fn opt_str_to_owned(o: Option<&str>) -> Option<String> {
    o.map(|s| s.to_owned())
}
/// ID3の任意のキーで文字列値を取得
fn id3_get_str(tag: &Tag, key: &str) -> Option<String> {
    let text = tag.get(key).and_then(|frame| frame.content().text());
    opt_str_to_owned(text)
}
/// ID3からリリース日を取得
fn id3_get_release_date(tag: &Tag) -> Result<Option<NaiveDate>, MP3Error> {
    let opt_year = tag.year();
    let opt_date = tag.get(KEY_DATE).and_then(|frame| frame.content().text());

    //両方Noneなら正常にNone
    //片方だけNoneならInvalid
    match opt_year {
        Some(year) => match opt_date {
            Some(date_str) => year_date_to_release_date(year, date_str),
            None => Err(MP3Error::InvalidReleaseDate {
                year: Some(year),
                tdat: None,
            }),
        },
        None => match opt_date {
            Some(date_str) => Err(MP3Error::InvalidReleaseDate {
                year: None,
                tdat: Some(date_str.to_string()),
            }),
            None => Ok(None),
        },
    }
}

/// ID3からメモを取得
fn id3_get_memo(tag: &Tag) -> Option<String> {
    let v = tag
        .comments()
        .filter(|c| c.description.is_empty())
        .map(|c| trim_null(&c.text))
        .collect::<Vec<_>>();

    if v.is_empty() {
        None
    } else {
        Some(v.join("\n"))
    }
}

/*
/// ID3から歌詞を取得
fn id3_get_lyrics(tag: &Tag) -> Option<String> {
    let v = tag
        .lyrics()
        .map(|c| trim_null(&c.text).replace("\r", "\n"))
        .collect::<Vec<_>>();

    if v.is_empty() {
        None
    } else {
        Some(v.join("\n"))
    }
}
*/

/// 文字列の末尾からnull文字を除去
fn trim_null(s: &str) -> &str {
    if let Some(stripped) = s.strip_suffix('\u{0}') {
        stripped
    } else {
        s
    }
}

/// 年・日付文字列をReleaseDateに変換
fn year_date_to_release_date(year: i32, date: &str) -> Result<Option<NaiveDate>, MP3Error> {
    if date.len() != 4 {
        return Err(MP3Error::InvalidReleaseDate {
            year: Some(year),
            tdat: Some(date.to_string()),
        });
    }

    let s = format!("{year}/{date}");
    match NaiveDate::parse_from_str(&s, "%Y/%d%m") {
        Ok(date) => Ok(Some(date)),
        Err(_) => Err(MP3Error::InvalidReleaseDate {
            year: Some(year),
            tdat: Some(date.to_string()),
        }),
    }
}
/// ID3タグにリリース日を設定
fn id3_set_release_date(tag: &mut Tag, date: &Option<NaiveDate>) {
    match date {
        Some(d) => {
            tag.set_year(d.year());
            let s = format!("{:02}{:02}", d.day(), d.month());
            tag.set_text(KEY_DATE, &s);
        }
        None => {
            tag.remove_year();
            tag.remove(KEY_DATE);
        }
    }
}

/// ID3タグにアートワークを設定
fn id3_set_artworks(tag: &mut Tag, artworks: Vec<TrackArtwork>) -> Result<(), MP3Error> {
    use id3::frame::{Picture, PictureType};

    //一旦全削除
    tag.remove_all_pictures();

    for artwork in artworks {
        //既に同じPictureTypeを登録していないか確認
        if tag
            .pictures()
            .any(|p| u8::from(p.picture_type) == artwork.picture_type)
        {
            return Err(MP3Error::PictureTypeDuplicated {
                type_num: artwork.picture_type,
            });
        }

        tag.add_frame(Picture {
            mime_type: artwork.picture.mime_type,
            picture_type: PictureType::Undefined(artwork.picture_type),
            description: artwork.description,
            data: artwork.picture.bytes,
        });
    }

    Ok(())
}

/// MP3 曲データ関連のエラー
#[derive(thiserror::Error, Debug)]
pub enum MP3Error {
    #[error(transparent)]
    Id3(#[from] id3::Error),

    /// ファイルIO汎用エラー
    #[error("{0}: {1}")]
    FileIoError(PathBuf, std::io::Error),

    #[error(transparent)]
    DurationError(#[from] MP3DurationError),

    #[error("{}", display_invalid_release_date(.year, .tdat))]
    InvalidReleaseDate {
        /// `id3::Tag::year() の値` (TYER)
        year: Option<i32>,

        /// TDAT
        tdat: Option<String>,
    },

    #[error("アートワークのPicture typeが重複しています: {type_num}")]
    PictureTypeDuplicated { type_num: u8 },
}

fn display_invalid_release_date(year: &Option<i32>, tdat: &Option<String>) -> String {
    let year = match year {
        Some(i) => Cow::Owned(i.to_string()),
        None => Cow::Borrowed("None"),
    };

    let tdat = tdat.as_deref().unwrap_or("None");

    format!("リリース日の値が不正です: year = {year}, TDAT = {tdat}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id3_get_release_date() -> anyhow::Result<()> {
        fn comm_valid(
            year: Option<&str>,
            date: Option<&str>,
            r: Option<NaiveDate>,
        ) -> anyhow::Result<()> {
            let mut tag = Tag::new();
            if let Some(y) = year {
                tag.set_text("TYER", y);
            }
            if let Some(d) = date {
                tag.set_text(KEY_DATE, d);
            }
            assert_eq!(id3_get_release_date(&tag)?, r);

            Ok(())
        }
        fn comm_invalid(year: Option<i32>, date: Option<&str>) {
            let mut tag = Tag::new();
            if let Some(y) = year {
                tag.set_text("TYER", y.to_string());
            }
            if let Some(d) = date {
                tag.set_text(KEY_DATE, d);
            }
            match id3_get_release_date(&tag).unwrap_err() {
                MP3Error::InvalidReleaseDate { year: r_year, tdat } => {
                    assert_eq!(r_year, year);
                    assert_eq!(tdat.as_deref(), date)
                }
                e => panic!("unknown error: {e}"),
            }
        }

        comm_valid(
            Some("2021"),
            Some("2403"),
            Some(NaiveDate::from_ymd_opt(2021, 3, 24).unwrap()),
        )?;
        comm_valid(
            Some("123"),
            Some("0706"),
            Some(NaiveDate::from_ymd_opt(123, 6, 7).unwrap()),
        )?;
        comm_invalid(Some(350), Some("219"));
        comm_invalid(Some(2001), Some("211"));
        comm_invalid(Some(2001), Some("11"));
        comm_invalid(Some(2003), Some("2902"));
        comm_invalid(Some(2003), Some("3013"));
        comm_invalid(Some(2003), None);
        comm_invalid(None, Some("1407"));
        comm_valid(None, None, None)?;

        Ok(())
    }

    #[test]
    fn test_id3_set_release_date() {
        let mut tag = Tag::new();

        id3_set_release_date(
            &mut tag,
            &Some(NaiveDate::from_ymd_opt(2021, 3, 16).unwrap()),
        );
        assert_eq!(tag.year(), Some(2021));
        assert_eq!(tag.get(KEY_DATE).unwrap().content().text(), Some("1603"));

        id3_set_release_date(
            &mut tag,
            &Some(NaiveDate::from_ymd_opt(198, 11, 3).unwrap()),
        );
        assert_eq!(tag.year(), Some(198));
        assert_eq!(tag.get(KEY_DATE).unwrap().content().text(), Some("0311"));

        id3_set_release_date(&mut tag, &None);
        assert!(tag.year().is_none());
        assert!(tag.get(KEY_DATE).is_none());
    }
}
