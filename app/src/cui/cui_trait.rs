use anyhow::Result;
use std::fmt::Arguments;

/// CUIの抽象化
pub trait Cui {
    /// 標準出力へ出力
    fn out(&self, args: Arguments);

    /// 標準出力へ出力(改行付き)
    fn outln(&self, args: Arguments) {
        self.out(args);
        self.out(format_args!("\n"));
    }

    /// 標準エラーへ出力
    fn err(&self, args: Arguments);

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
    fn input_case(&self, cases: &[char], message: &str) -> Result<char>;
}
