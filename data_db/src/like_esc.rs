//! LIKE文エスケープ関係

/**
 * LIKE文でエスケープされるべき文字
 *
 * ($はエスケープ対象だが、エスケープ判定では無視する)
 */
const LIKE_ESC_CHARS: [char; 5] = ['$', '%', '_', '[', ']'];

/// 文字列をLIKE文の値として使うために、エスケープが必要か調べる
/// # Arguments
/// - s: 対象の文字列
/// # Returns
/// trueならエスケープ必要
pub fn is_need(s: &str) -> bool {
    //エスケープ対象文字の配列で指定された文字がないか探す
    //(ただし先頭の$だけならエスケープ不要)
    LIKE_ESC_CHARS[1..].iter().any(|it| s.contains(*it))
}

/// 文字列をLIKE文の値として使えるようエスケープ
///
/// エスケープ文字には$を使用する
///
/// # Arguments
/// - s: エスケープ元の文字列
/// # Returns
/// エスケープされた文字列
pub fn escape(s: &str) -> String {
    let mut str_var = String::from(s);

    for chr in LIKE_ESC_CHARS {
        let mut to = String::from('$');
        to.push(chr);
        str_var = str_var.replace(chr, &to);
    }

    str_var
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_need() {
        assert!(!is_need(""));
        assert!(!is_need("test"));
        assert!(!is_need("te$st"));
        assert!(is_need("te$s%t"));
        assert!(is_need("%test"));
    }

    #[test]
    fn test_escape() {
        assert_eq!(&escape("te$s%t"), "te$$s$%t");
        assert_eq!(&escape("%test"), "$%test");
    }
}
