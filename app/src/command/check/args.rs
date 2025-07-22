use anyhow::Result;
use murack_core_domain::path::LibPathStr;

/// checkコマンドの引数
#[derive(Debug, PartialEq, Clone)]
pub struct Args {
    /// 確認対象のパス
    ///
    /// 未入力の場合はroot(ライブラリ全体をチェックする)
    pub path: LibPathStr,

    /// DAPのファイル内容を無視するか
    ///
    /// trueなら、PC間とDAP間でファイル内容を比較しない
    /// (一致として扱う)
    pub ignore_dap_content: bool,
}

impl Args {
    /// コマンドの引数を解析
    pub fn parse(command_line: &[String]) -> Result<Self> {
        let mut path = LibPathStr::root();
        let mut ignore_dap_content = false;

        for unit in command_line.iter() {
            if unit == "-i" {
                ignore_dap_content = true;
            }
            //オプション以外はパスと解釈
            else {
                path = unit.clone().into();
            }
        }

        Ok(Args {
            path,
            ignore_dap_content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() -> anyhow::Result<()> {
        assert_eq!(
            Args::parse(&["tgt".to_owned()])?,
            Args {
                path: "tgt".to_owned().into(),
                ignore_dap_content: false,
            }
        );
        assert_eq!(
            Args::parse(&["tgt/file".to_owned(), "-i".to_owned()])?,
            Args {
                path: "tgt/file".to_owned().into(),
                ignore_dap_content: true,
            }
        );
        assert_eq!(
            Args::parse(&["-i".to_owned()])?,
            Args {
                path: LibPathStr::root(),
                ignore_dap_content: true,
            }
        );
        assert_eq!(
            Args::parse(&["-i".to_owned(), "tgt/file".to_owned()])?,
            Args {
                path: "tgt/file".to_owned().into(),
                ignore_dap_content: true,
            }
        );
        assert_eq!(
            Args::parse(&[])?,
            Args {
                path: LibPathStr::root(),
                ignore_dap_content: false,
            }
        );

        Ok(())
    }
}
