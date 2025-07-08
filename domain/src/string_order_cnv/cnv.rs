//! cnv関数

use super::map_define::*;

/// 値の読み方を、ソート用に自動変換して作成
/// # Arguments
/// - base_str: baseStr 変換元の値
/// # Returns
/// ソート用に変換された値
pub fn cnv(base_str: &str) -> String {
    let mut strb = String::new();

    //変換元文字の各文字について処理
    let chars: Vec<char> = base_str.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let current = chars[i];
        //変換マップにあれば変換
        if let Some(nml_value) = CHAR_ORDER_MAP.get(&current) {
            strb.push(*nml_value);
        }
        //半角カタカナ変換マップにあった場合
        else if let Some(daku_obj) = HAN_KANA_MAP.get(&current) {
            match chars.get(i + 1) {
                //次の文字が濁点なら、まとめて濁音とする
                Some('ﾞ') => {
                    strb.push(daku_obj.daku);
                    i += 1;
                }
                //次の文字が半濁点なら、まとめて半濁音とする
                Some('ﾟ') => match daku_obj.handaku {
                    Some(c) => {
                        strb.push(c);
                        i += 1;
                    }
                    //半濁音の変換先が未定義なら、通常文字とする
                    None => {
                        strb.push(daku_obj.nml);
                    }
                },
                //濁点でも半濁点でもないか、次の文字がなければ、通常文字に変換
                _ => {
                    strb.push(daku_obj.nml);
                }
            }
        }
        //いずれの変換マップにもなければ変換しない
        else {
            strb.push(current);
        }

        i += 1;
    }

    strb
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("アイうｴおがギグげｺﾞハピﾌﾍﾞﾎﾟ漢字", "あいうえおがぎぐげごはぴふべぽ漢字" ; "jp")]
    #[test_case("AbcdE efGhI", "abcde efghi" ; "ascii")]
    #[test_case("ｶﾞｷﾞｸﾞ", "がぎぐ" ; "ends_with_daku")]
    #[test_case("ﾊﾞﾋﾞﾌﾞ", "ばびぶ" ; "ends_with_daku_maybe_handaku")]
    #[test_case("ﾊﾟﾋﾟﾌﾟ", "ぱぴぷ" ; "ends_with_handaku")]
    #[test_case("ｻｼｽ", "さしす" ; "ends_with_normal_maybe_daku")]
    #[test_case("ﾊﾋﾌ", "はひふ" ; "ends_with_normal_maybe_handaku")]
    #[test_case("ﾅﾞﾅ゜ﾞﾅﾞ", "なﾞな゜ﾞなﾞ" ; "invalid_daku")]
    fn text_cnv(input: &str, expect: &str) {
        assert_eq!(&cnv(input), expect);
    }
}
