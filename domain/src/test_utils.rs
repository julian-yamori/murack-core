//順番不問でスライス内容の一致を確認
pub fn assert_eq_not_orderd<T: PartialEq>(left: &[T], right: &[T]) {
    assert_eq!(left.len(), right.len());
    for path in right {
        assert!(left.iter().any(|p| p == path));
    }
}
