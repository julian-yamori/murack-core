use anyhow::Result;
use murack_core_domain::path::LibPathStr;

/// checkコマンドの引数
#[derive(Debug, PartialEq, Clone)]
pub struct CommandCheckArgs {
    /// 確認対象のパス
    ///
    /// None の場合はライブラリ全体をチェックする。
    pub path: Option<LibPathStr>,

    /// DAPのファイル内容を無視するか
    ///
    /// trueなら、PC間とDAP間でファイル内容を比較しない
    /// (一致として扱う)
    pub ignore_dap_content: bool,
}

impl CommandCheckArgs {
    /// コマンドライン引数から解析
    pub fn parse(command_line: &[String]) -> Result<Self> {
        let mut path: Option<LibPathStr> = None;
        let mut ignore_dap_content = false;

        for unit in command_line.iter() {
            if unit == "-i" {
                ignore_dap_content = true;
            }
            //オプション以外はパスと解釈
            else {
                path = Some(unit.clone().try_into()?);
            }
        }

        Ok(CommandCheckArgs {
            path,
            ignore_dap_content,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_parse() -> anyhow::Result<()> {
        assert_eq!(
            CommandCheckArgs::parse(&["tgt".to_owned()])?,
            CommandCheckArgs {
                path: Some(LibPathStr::from_str("tgt")?),
                ignore_dap_content: false,
            }
        );
        assert_eq!(
            CommandCheckArgs::parse(&["tgt/file".to_owned(), "-i".to_owned()])?,
            CommandCheckArgs {
                path: Some(LibPathStr::from_str("tgt/file")?),
                ignore_dap_content: true,
            }
        );
        assert_eq!(
            CommandCheckArgs::parse(&["-i".to_owned()])?,
            CommandCheckArgs {
                path: None,
                ignore_dap_content: true,
            }
        );
        assert_eq!(
            CommandCheckArgs::parse(&["-i".to_owned(), "tgt/file".to_owned()])?,
            CommandCheckArgs {
                path: Some(LibPathStr::from_str("tgt/file")?),
                ignore_dap_content: true,
            }
        );
        assert_eq!(
            CommandCheckArgs::parse(&[])?,
            CommandCheckArgs {
                path: None,
                ignore_dap_content: false,
            }
        );

        Ok(())
    }
}
