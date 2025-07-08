use std::path::PathBuf;

/// アプリケーション自体の引数
#[derive(Debug, PartialEq)]
pub struct AppArgs {
    /// コンフィグファイルのパス
    ///
    /// `-c`で指定
    pub config_path: Option<PathBuf>,

    /// サブコマンド名
    pub sub_command: Option<String>,

    /// サブコマンド引数
    pub sub_args: Vec<String>,
}

impl AppArgs {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Self {
        let mut parsed = Self {
            config_path: None,
            sub_command: None,
            sub_args: vec![],
        };

        //実行ファイル名は読み飛ばし
        args.next();

        while let Some(a) = args.next() {
            match &*a {
                //コンフィグパス指定
                "-c" => {
                    let o = args.next();
                    parsed.config_path = o.map(PathBuf::from);
                }
                //それ以外の場合、サブコマンド名とみなす
                _ => {
                    parsed.sub_command = Some(a);
                    parsed.sub_args = args.collect();
                    break;
                }
            }
        }

        parsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        fn si<'a>(
            a: &'a [&str],
        ) -> core::iter::Map<core::slice::Iter<'a, &'a str>, fn(&&str) -> String> {
            a.iter().map(|s| s.to_string())
        }

        pretty_assertions::assert_eq!(
            AppArgs::parse(si(&["exe"][..])),
            AppArgs {
                config_path: None,
                sub_command: None,
                sub_args: vec![]
            }
        );
        pretty_assertions::assert_eq!(
            AppArgs::parse(si(&["exe", "check"][..])),
            AppArgs {
                config_path: None,
                sub_command: Some("check".to_owned()),
                sub_args: vec![]
            }
        );
        pretty_assertions::assert_eq!(
            AppArgs::parse(si(&["exe", "add", "hoge/fuga.flac"][..])),
            AppArgs {
                config_path: None,
                sub_command: Some("add".to_owned()),
                sub_args: vec!["hoge/fuga.flac".to_owned()]
            }
        );
        pretty_assertions::assert_eq!(
            AppArgs::parse(si(&["exe", "check", "-i", "hoge/fuga.flac"][..])),
            AppArgs {
                config_path: None,
                sub_command: Some("check".to_owned()),
                sub_args: vec!["-i".to_owned(), "hoge/fuga.flac".to_owned()]
            }
        );
        pretty_assertions::assert_eq!(
            AppArgs::parse(si(
                &["exe", "-c", "config.toml", "add", "hoge/fuga.flac"][..]
            )),
            AppArgs {
                config_path: Some("config.toml".into()),
                sub_command: Some("add".to_owned()),
                sub_args: vec!["hoge/fuga.flac".to_owned()]
            }
        );
        pretty_assertions::assert_eq!(
            AppArgs::parse(si(&["exe", "-c", "add", "hoge/fuga.flac"][..])),
            AppArgs {
                config_path: Some("add".into()),
                sub_command: Some("hoge/fuga.flac".to_owned()),
                sub_args: vec![]
            }
        );
    }
}
