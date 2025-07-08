use super::Cui;
use anyhow::Result;
use std::{
    fmt::Arguments,
    io::{self, Write},
};

/// 標準入出力を使用するCui trait実装
pub struct StdCui {}

impl Cui for StdCui {
    /// 標準出力へ出力
    fn out(&self, args: Arguments) {
        let mut stdout = io::stdout();
        stdout.write_fmt(args).unwrap();
        stdout.flush().unwrap();
    }

    /// エラーを出力
    fn err(&self, args: Arguments) {
        let mut stderr = io::stderr();
        stderr.write_fmt(args).unwrap();
        stderr.flush().unwrap();
    }

    /// 選択肢を示して、文字を入力させる。
    ///
    /// 選択肢以外が入力されたら、メッセージを再表示し、再入力を促す。
    /// 入力された文字の大文字・小文字は区別しない
    ///
    /// # Arguments
    /// - cases : 選択肢の文字(小文字、もしくは数字)
    /// - message: 入力前に表示するメッセージ
    ///
    /// # Returns
    /// 入力された文字(casesのうちのいずれか)
    fn input_case(&self, cases: &[char], message: &str) -> Result<char> {
        loop {
            print!("{}", message);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let chars: Vec<char> = input.trim().to_lowercase().chars().collect();
            if chars.len() == 1 {
                let input_lower = chars[0];

                //入力成功し、選択肢内の文字ならループを抜ける
                if cases.iter().any(|c| *c == input_lower) {
                    return Ok(input_lower);
                }
            }
        }
    }
}
