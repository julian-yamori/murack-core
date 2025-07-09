//! MP3フォーマット取扱

use std::{
    fs::File,
    io::{BufReader, Seek},
    path::Path,
};

use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use id3::{Tag, TagLike};

use crate::{
    Error,
    audio_meta::{AudioMetaData, AudioMetaDataEntry, AudioPicture, AudioPictureEntry},
};

const KEY_COMPOSER: &str = "TCOM";
const KEY_DATE: &str = "TDAT";

/// ファイルからメタデータを読み込み
///
/// # Arguments
/// - path: オーディオファイルの絶対パス
/// # Returns
/// オーディオファイルのメタデータ
pub fn read(path: &Path) -> Result<AudioMetaData> {
    let file = File::open(path).map_err(|e| Error::FileIoError(path.to_owned(), e))?;
    let mut reader = BufReader::new(file);

    let tag = Tag::read_from(&mut reader)?;

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
            .map(|picture| AudioPicture {
                bytes: picture.data.clone(),
                mime_type: picture.mime_type.clone(),
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
/// - song: 書き込む曲の情報
pub fn overwrite(
    path: &Path,
    song: &AudioMetaDataEntry,
    artworks: &[AudioPictureEntry],
) -> Result<()> {
    let mut tag = Tag::read_from_path(&path)?;

    match song.title {
        Some(v) => tag.set_title(v),
        None => tag.remove_title(),
    }
    match song.artist {
        Some(v) => tag.set_artist(v),
        None => tag.remove_artist(),
    }
    match song.album {
        Some(v) => tag.set_album(v),
        None => tag.remove_album(),
    }
    match song.genre {
        Some(v) => tag.set_genre(v),
        None => tag.remove_genre(),
    }
    match song.album_artist {
        Some(v) => tag.set_album_artist(v),
        None => tag.remove_album_artist(),
    }
    match song.composer {
        Some(v) => tag.set_text(KEY_COMPOSER, v),
        None => {
            tag.remove(KEY_COMPOSER);
        }
    }
    match song.track_number {
        Some(v) => tag.set_track(v as u32),
        None => tag.remove_track(),
    }
    match song.track_max {
        Some(v) => tag.set_total_tracks(v as u32),
        None => tag.remove_total_tracks(),
    }
    match song.disc_number {
        Some(v) => tag.set_disc(v as u32),
        None => tag.remove_disc(),
    }
    match song.disc_max {
        Some(v) => tag.set_total_discs(v as u32),
        None => tag.remove_total_discs(),
    }

    id3_set_release_date(&mut tag, &song.release_date);

    tag.remove_comment(Some(""), None);
    if let Some(s) = song.memo {
        tag.add_frame(id3::frame::Comment {
            lang: "".to_owned(),
            description: "".to_owned(),
            text: s.to_owned(),
        });
    }
    /*
    tag.remove_all_lyrics();
    if let Some(s) = &song.lyrics {
        tag.add_lyrics(id3::frame::Lyrics {
            lang: "".to_owned(),
            description: "".to_owned(),
            text: s.to_owned(),
        });
    }
    */

    id3_set_artworks(&mut tag, artworks)?;

    tag.write_to_path(path, id3::Version::Id3v23)?;
    Ok(())
}

/// MP3から再生時間を読み込み
///
/// # Arguments
/// - reader: 読み込み元のファイルリーダー
/// - path: 読み込むファイルのパス（エラー情報用）
fn read_duration(reader: &mut BufReader<File>, path: &Path) -> Result<u32> {
    let offset = reader
        .stream_position()
        .map_err(|e| Error::FileIoError(path.to_owned(), e))?;

    match mp3_duration::from_read(reader) {
        Ok(d) => Ok(d.as_millis() as u32),
        Err(mut e) => {
            e.offset += offset as usize;
            Err(Error::InvalidDuration {
                msg: format!("* MP3 Duration Error: \n{}", e),
            }
            .into())
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
fn id3_get_release_date(tag: &Tag) -> Result<Option<NaiveDate>> {
    let opt_year = tag.year();
    let opt_date = tag.get(KEY_DATE).and_then(|frame| frame.content().text());

    //両方Noneなら正常にNone
    //片方だけNoneならInvalid
    match opt_year {
        Some(year) => match opt_date {
            Some(date_str) => year_date_to_release_date(year, date_str),
            None => Err(Error::InvalidReleaseDate {
                value_info: format!("TYER: {}, TDAT: None", year),
            }
            .into()),
        },
        None => match opt_date {
            Some(date_str) => Err(Error::InvalidReleaseDate {
                value_info: format!("TYER: None, TDAT: {}", date_str),
            }
            .into()),
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
fn year_date_to_release_date(year: i32, date: &str) -> Result<Option<NaiveDate>> {
    if date.len() != 4 {
        return Err(Error::InvalidReleaseDate {
            value_info: format!("TYER: {}, TDAT: {}", year, date),
        }
        .into());
    }

    let s = format!("{}/{}", year, date);
    match NaiveDate::parse_from_str(&s, "%Y/%d%m") {
        Ok(date) => Ok(Some(date)),
        Err(_) => Err(Error::InvalidReleaseDate {
            value_info: format!("TYER: {}, TDAT: {}", year, date),
        }
        .into()),
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
fn id3_set_artworks(tag: &mut Tag, artworks: &[AudioPictureEntry]) -> Result<()> {
    use id3::frame::{Picture, PictureType};

    //一旦全削除
    tag.remove_all_pictures();

    for artwork in artworks {
        //既に同じPictureTypeを登録していないか確認
        if tag
            .pictures()
            .any(|p| u8::from(p.picture_type) == artwork.picture_type)
        {
            return Err(Error::Id3PictureTypeDuplicated {
                type_num: artwork.picture_type,
            }
            .into());
        }

        tag.add_frame(Picture {
            mime_type: artwork.mime_type.to_owned(),
            picture_type: PictureType::Undefined(artwork.picture_type),
            description: artwork.description.to_owned(),
            data: artwork.bytes.to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id3_get_release_date() {
        fn comm_valid(year: Option<&str>, date: Option<&str>, r: Option<NaiveDate>) {
            let mut tag = Tag::new();
            if let Some(y) = year {
                tag.set_text("TYER", y);
            }
            if let Some(d) = date {
                tag.set_text(KEY_DATE, d);
            }
            assert_eq!(id3_get_release_date(&tag).unwrap(), r);
        }
        fn comm_invalid(year: Option<&str>, date: Option<&str>, value_info: &str) {
            let mut tag = Tag::new();
            if let Some(y) = year {
                tag.set_text("TYER", y);
            }
            if let Some(d) = date {
                tag.set_text(KEY_DATE, d);
            }
            assert!(
                match id3_get_release_date(&tag).unwrap_err().downcast_ref() {
                    Some(Error::InvalidReleaseDate { value_info: vi }) => vi == value_info,
                    _ => false,
                }
            )
        }

        comm_valid(
            Some("2021"),
            Some("2403"),
            Some(NaiveDate::from_ymd(2021, 3, 24)),
        );
        comm_valid(
            Some("123"),
            Some("0706"),
            Some(NaiveDate::from_ymd(123, 6, 7)),
        );
        comm_invalid(Some("350"), Some("219"), "TYER: 350, TDAT: 219");
        comm_invalid(Some("2001"), Some("211"), "TYER: 2001, TDAT: 211");
        comm_invalid(Some("2001"), Some("11"), "TYER: 2001, TDAT: 11");
        comm_invalid(Some("2003"), Some("2902"), "TYER: 2003, TDAT: 2902");
        comm_invalid(Some("2003"), Some("3013"), "TYER: 2003, TDAT: 3013");
        comm_invalid(Some("2003"), None, "TYER: 2003, TDAT: None");
        comm_invalid(None, Some("1407"), "TYER: None, TDAT: 1407");
        comm_valid(None, None, None);
    }

    #[test]
    fn test_id3_set_release_date() {
        let mut tag = Tag::new();

        id3_set_release_date(&mut tag, &Some(NaiveDate::from_ymd(2021, 3, 16)));
        assert_eq!(tag.year(), Some(2021));
        assert_eq!(tag.get(KEY_DATE).unwrap().content().text(), Some("1603"));

        id3_set_release_date(&mut tag, &Some(NaiveDate::from_ymd(198, 11, 3)));
        assert_eq!(tag.year(), Some(198));
        assert_eq!(tag.get(KEY_DATE).unwrap().content().text(), Some("0311"));

        id3_set_release_date(&mut tag, &None);
        assert!(tag.year().is_none());
        assert!(tag.get(KEY_DATE).is_none());
    }
}
