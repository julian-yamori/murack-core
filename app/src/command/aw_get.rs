use crate::{Config, Error, cui::Cui};

use anyhow::{Context, Result, anyhow};
use domain::{FileLibraryRepository, path::LibSongPath};

use std::{
    fs,
    path::{Path, PathBuf},
};

/// aw-getコマンド
///
/// 曲ファイルからアートワークを取得する
pub struct CommandArtworkGet<CUI, FR>
where
    CUI: Cui,
    FR: FileLibraryRepository,
{
    args: Args,
    config: Config,
    cui: CUI,
    file_library_repository: FR,
}

impl<CUI, FR> CommandArtworkGet<CUI, FR>
where
    CUI: Cui,
    FR: FileLibraryRepository,
{
    pub fn new(
        command_line: &[String],
        config: Config,
        cui: CUI,
        file_library_repository: FR,
    ) -> Result<Self> {
        Ok(Self {
            args: parse_args(command_line)?,
            config,
            cui,
            file_library_repository,
        })
    }

    /// このコマンドを実行
    pub fn run(&self) -> Result<()> {
        //出力先パス(未指定なら曲パスを使用)
        //todo WalkBase1では拡張子を省いたりargsの規約で省略させたりするが、
        //WalkBase2ではwith_extentionでの置き換え
        //名前をdest_pathあたりにした方がわかりやすいか？
        let artwork_path = match &self.args.artwork_path {
            Some(p) => p.clone(),
            None => self.args.song_path.abs(&self.config.pc_lib),
        };

        //指定された曲ファイルを解析
        let audio_meta = self
            .file_library_repository
            .read_audio_meta(&self.config.pc_lib, &self.args.song_path)?;

        //各アートワークを出力
        let artworks_len = audio_meta.artworks.len();
        for (idx, artwork) in audio_meta.artworks.iter().enumerate() {
            //出力先ファイル名を作成
            let out_path = make_output_path(&artwork_path, &artwork.mime_type, idx, artworks_len)?;

            //既にファイルが存在するなら失敗
            if out_path.exists() {
                return Err(anyhow!(
                    "出力先にファイルが存在しています: {}",
                    out_path.display()
                ));
            }

            fs::write(&out_path, &artwork.bytes)
                .with_context(|| format!("ファイルの書き込みに失敗: {}", out_path.display()))?;

            cui_outln!(self.cui, "=> {}", out_path.display());
        }

        cui_outln!(self.cui);

        Ok(())
    }
}

/// アートワークの出力先ファイルパスを作成
///
/// # Arguments
/// - dest: 大まかな出力先パス指定
/// - mime_type: 出力する画像のmime_type
/// - current_idx: 何番目のアートワーク画像か
/// - all_count: 出力するアートワーク画像総数
fn make_output_path(
    dest: &Path,
    mime_type: &str,
    current_idx: usize,
    all_count: usize,
) -> Result<PathBuf> {
    //拡張子をアートワーク画像のものに置き換え
    let ext = match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        s => return Err(media::Error::UnsupportedArtworkFmt { fmt: s.to_owned() }.into()),
    };

    let old_stem = dest
        .file_stem()
        .with_context(|| format!("ファイル名の取得に失敗: {}", dest.display()))?
        .to_str()
        .with_context(|| format!("UTF-8への変換に失敗: {}", dest.display()))?;

    //総数が2個以上なら連番付け
    let new_name = if all_count > 1 {
        format!("{old_stem}_{current_idx}.{ext}")
    } else {
        format!("{old_stem}.{ext}")
    };

    Ok(dest.with_file_name(new_name))
}

/// コマンドの引数
struct Args {
    /// 曲ファイルのパス
    song_path: LibSongPath,

    /// 画像ファイルの保存先パス
    ///
    /// 拡張子は不要。
    /// アートワークの種類により、自動で付与される。
    ///
    /// Noneの場合、songPathを使用する。
    artwork_path: Option<PathBuf>,
}

/// コマンドの引数を解析
fn parse_args(command_line: &[String]) -> Result<Args> {
    match command_line {
        [song, artwork, ..] => Ok(Args {
            song_path: LibSongPath::new(song),
            artwork_path: Some(artwork.into()),
        }),
        [song] => Ok(Args {
            song_path: LibSongPath::new(song),
            artwork_path: None,
        }),
        [] => Err(Error::InvalidCommandArgument {
            msg: "audio file path is not specified.".to_owned(),
        }
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("hoge/fuga.flac", "image/jpeg", 0, 1, "hoge/fuga.jpg"; "normal")]
    #[test_case("fuga.mp3", "image/bmp", 0, 1, "fuga.bmp"; "root")]
    #[test_case("hoge/fuga", "image/jpeg", 0, 1, "hoge/fuga.jpg"; "no_ext")]
    #[test_case("fuga", "image/png", 0, 1, "fuga.png"; "root_no_ext")]
    #[test_case("hoge/fuga.flac", "image/jpeg", 0, 3, "hoge/fuga_0.jpg"; "number")]
    #[test_case("hoge/fuga", "image/gif", 1, 3, "hoge/fuga_1.gif"; "number_no_ext")]
    fn test_make_output_path(
        dest: &str,
        mime_type: &str,
        current_idx: usize,
        all_count: usize,
        expect: &str,
    ) -> anyhow::Result<()> {
        assert_eq!(
            make_output_path(&PathBuf::from(dest), mime_type, current_idx, all_count)?,
            PathBuf::from(expect)
        );

        Ok(())
    }
}
