//! プレイリストファイル名の生成関係

use crate::command::playlist::PLAYLIST_EXT;

/// プレイリストファイル名を作るために必要な諸々の値
pub struct FileNameContext<'name> {
    /// DAP に保存するプレイリストの総数
    pub all_count: u32,

    /// 次に生成するプレイリストが、全体で何番目か
    pub offset_of_whole: u32,

    /// 次に生成するプレイリストの親の、プレイリスト名の Vec
    pub parent_names: Vec<&'name str>,
}

impl FileNameContext<'_> {
    pub fn new(all_count: u32) -> Self {
        Self {
            offset_of_whole: 1,
            all_count,
            parent_names: vec![],
        }
    }

    /// ファイル名先頭の数値部分の桁数
    pub fn number_digit(&self) -> u32 {
        let mut num = self.all_count;

        let mut digit = 0;
        while num > 0 {
            num /= 10;
            digit += 1;
        }

        digit
    }
}

pub fn build_file_name(playlist_name: &str, context: &FileNameContext) -> String {
    let digit = context.number_digit();

    //番号を付ける
    //TODO 書式つかってもっときれいに実装できそう
    let mut buf = context.offset_of_whole.to_string();
    //総数の桁数に応じて0埋め
    for _ in 0..(digit - buf.len() as u32) {
        buf.insert(0, '0');
    }

    //親がいるなら追加
    if !context.parent_names.is_empty() {
        let joined_names = context.parent_names.join("-");

        buf = format!("{buf}-{joined_names}");
    }

    format!("{buf}-{playlist_name}.{PLAYLIST_EXT}")
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(1, 1 ; "1")]
    #[test_case(9, 1 ; "9")]
    #[test_case(10, 2 ; "10")]
    #[test_case(99, 2 ; "99")]
    #[test_case(100, 3 ; "100")]
    fn test_get_digit(input: u32, expect: u32) {
        let context = FileNameContext::new(input);
        assert_eq!(context.number_digit(), expect);
    }

    #[test_case("plist", &[], 3, 12, "03-plist.m3u" ; "root")]
    #[test_case("plist", &["parent"], 3, 8, "3-parent-plist.m3u" ; "one_parent_one_digit")]
    #[test_case("plist", &["parent", "2"], 45, 100, "045-parent-2-plist.m3u" ; "two_parents_three_digit")]
    #[test_case("plist-pl", &["parent"], 5, 999, "005-parent-plist-pl.m3u" ; "hyphen_name")]
    fn test_playlist_to_file_name(
        name: &str,
        parent_names: &[&str],
        offset_of_whole: u32,
        all_count: u32,
        expect: &str,
    ) {
        let context = FileNameContext {
            offset_of_whole,
            all_count,
            parent_names: parent_names.to_vec(),
        };
        assert_eq!(&build_file_name(name, &context), expect);
    }
}
