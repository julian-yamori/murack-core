use super::{LibDirPath, LibTrackPath};
use crate::Error;
use anyhow::Result;

/// 曲ファイルを示す相対パス
#[derive(Debug, PartialEq, Clone)]
pub struct RelativeTrackPath(String);

impl RelativeTrackPath {
    /// 曲パスと親パスから、相対パスインスタンスを作成
    ///
    /// # Arguments
    /// - track: 曲のパス
    /// - parent: trackの親ディレクトリのパス
    ///
    /// # Returns
    /// parentからみたtrackの相対パス
    pub fn from_track_and_parent(track: &LibTrackPath, parent: &LibDirPath) -> Result<Self> {
        let parent_str = parent.as_str();
        let track_str = track.as_str();

        //targetがparentで始まっているか確認
        if !track_str.starts_with(parent_str) {
            return Err(Error::GetRelativePathFailed {
                track: track.to_owned(),
                parent: parent.to_owned(),
            }
            .into());
        }

        Ok(Self(track_str[parent_str.len()..].to_owned()))
    }

    /// LibDirPathと連結し、LibTrackPathを生成
    pub fn concat_lib_dir(&self, parent: &LibDirPath) -> LibTrackPath {
        let mut s = parent.as_str().to_owned();
        s.push_str(&self.0);
        LibTrackPath::new(s)
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
    fn test_from_track_and_parent_valid(
        track: &str,
        parent: &str,
        expect: &str,
    ) -> anyhow::Result<()> {
        assert_eq!(
            RelativeTrackPath::from_track_and_parent(
                &LibTrackPath::new(track),
                &LibDirPath::new(parent)
            )?,
            RelativeTrackPath(expect.to_owned())
        );

        Ok(())
    }

    #[test]
    fn test_from_track_and_parent_invalid() {
        let track = LibTrackPath::new("hoge/fuga.flac");
        let parent = LibDirPath::new("piyo");

        let err = RelativeTrackPath::from_track_and_parent(&track, &parent).unwrap_err();
        match err.downcast_ref() {
            Some(Error::GetRelativePathFailed {
                track: e_track,
                parent: e_parent,
            }) => {
                assert_eq!(e_track, &track);
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
            RelativeTrackPath(rel.to_owned()).concat_lib_dir(&LibDirPath::new(parent)),
            LibTrackPath::new(expect)
        )
    }
}
