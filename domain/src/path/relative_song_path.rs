use super::{LibDirPath, LibSongPath};
use crate::Error;
use anyhow::Result;

/// 曲ファイルを示す相対パス
#[derive(Debug, PartialEq, Clone)]
pub struct RelativeSongPath(String);

impl RelativeSongPath {
    /// 曲パスと親パスから、相対パスインスタンスを作成
    ///
    /// # Arguments
    /// - song: 曲のパス
    /// - parent: songの親ディレクトリのパス
    ///
    /// # Returns
    /// parentからみたsongの相対パス
    pub fn from_song_and_parent(song: &LibSongPath, parent: &LibDirPath) -> Result<Self> {
        let parent_str = parent.as_str();
        let song_str = song.as_str();

        //targetがparentで始まっているか確認
        if !song_str.starts_with(parent_str) {
            return Err(Error::GetRelativePathFailed {
                song: song.to_owned(),
                parent: parent.to_owned(),
            }
            .into());
        }

        Ok(Self(song_str[parent_str.len()..].to_owned()))
    }

    /// LibDirPathと連結し、LibSongPathを生成
    pub fn concat_lib_dir(&self, parent: &LibDirPath) -> LibSongPath {
        let mut s = parent.as_str().to_owned();
        s.push_str(&self.0);
        LibSongPath::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("hoge/fuga.flac", "hoge", "fuga.flac" ; "1")]
    #[test_case("hoge/fuga/piyo.flac", "hoge", "fuga/piyo.flac" ; "2")]
    #[test_case("hoge/fuga/piyo.flac", "hoge/fuga", "piyo.flac" ; "3")]
    #[test_case("piyo.flac", "", "piyo.flac" ; "root")]
    #[test_case("hoge/piyo.flac", "", "hoge/piyo.flac" ; "in_dir_from_root")]
    fn test_from_song_and_parent_valid(song: &str, parent: &str, expect: &str) {
        assert_eq!(
            RelativeSongPath::from_song_and_parent(
                &LibSongPath::new(song),
                &LibDirPath::new(parent)
            )
            .unwrap(),
            RelativeSongPath(expect.to_owned())
        )
    }

    #[test]
    fn test_from_song_and_parent_invalid() {
        let song = LibSongPath::new("hoge/fuga.flac");
        let parent = LibDirPath::new("piyo");

        let err = RelativeSongPath::from_song_and_parent(&song, &parent).unwrap_err();
        match err.downcast_ref() {
            Some(Error::GetRelativePathFailed {
                song: e_song,
                parent: e_parent,
            }) => {
                assert_eq!(e_song, &song);
                assert_eq!(e_parent, &parent);
            }
            _ => panic!("{}", err),
        }
    }

    #[test_case("fuga.flac", "hoge", "hoge/fuga.flac" ; "normal")]
    #[test_case("piyo/fuga.flac", "hoge", "hoge/piyo/fuga.flac" ; "rel_in_dir")]
    #[test_case("piyo/fuga.flac", "", "piyo/fuga.flac" ; "root_parent")]
    fn test_concat_lib_dir(rel: &str, parent: &str, expect: &str) {
        assert_eq!(
            RelativeSongPath(rel.to_owned()).concat_lib_dir(&LibDirPath::new(parent)),
            LibSongPath::new(expect)
        )
    }
}
