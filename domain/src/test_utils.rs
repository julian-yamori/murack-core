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

    for path in right {
        assert!(
            left.iter().any(|p| p == path),
            "assert_eq_not_orderd : left に {path} が含まれるか確認"
        );
    }
}
