use std::fmt::Display;

//順番不問でスライス内容の一致を確認
pub fn assert_eq_not_orderd<T>(left: &[T], right: &[T])
where
    T: PartialEq + Display,
{
    assert_eq!(
        left.len(),
        right.len(),
        "assert_eq_not_orderd : スライスの長さが一致するか確認 : {}, {}",
        left.len(),
        right.len()
    );

    for item in right {
        assert_eq!(
            left.iter().filter(|i| *i == item).count(),
            right.iter().filter(|i| *i == item).count(),
            "assert_eq_not_orderd : {item} の数が一致するか確認"
        );
    }
}

#[cfg(test)]
mod assert_eq_not_orderd_tests {
    use test_case::test_case;

    use super::*;

    #[test_case(&[1,2,3], &[1,2,3])]
    #[test_case(&[1,2,3], &[2,1,3])]
    #[test_case(&[], &[])]
    #[test_case(&[1], &[1])]
    fn should_success(left: &[i32], right: &[i32]) {
        assert_eq_not_orderd(left, right)
    }

    mod should_panics {
        use super::*;

        #[test]
        #[should_panic]
        fn length_not_matches() {
            assert_eq_not_orderd(&[1, 2, 3], &[1, 2])
        }

        #[test]
        #[should_panic]
        fn right_is_empty() {
            assert_eq_not_orderd(&[1, 2, 3], &[])
        }

        #[test]
        #[should_panic]
        fn left_is_empty() {
            assert_eq_not_orderd(&[], &[1, 2, 3])
        }

        #[test]
        #[should_panic]
        fn item_not_equals() {
            assert_eq_not_orderd(&[1, 2, 3], &[1, 99, 3])
        }

        #[test]
        #[should_panic]
        fn item_count_not_matches() {
            assert_eq_not_orderd(&[1, 1, 1, 2, 2], &[1, 1, 2, 2, 2])
        }
    }
}
