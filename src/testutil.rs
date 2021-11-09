#![cfg(test)]
#![allow(dead_code)]
pub(crate) mod mock_print;

pub(crate) fn are_slices_eq<T: PartialEq>(s1: &[T], s2: &[T]) -> bool {
    if s1.len() != s2.len() {
        return false;
    }

    // https://stackoverflow.com/a/29504547
    let len = s1.len();
    s1.iter().zip(s2).filter(|&(a, b)| a == b).count() == len
}

macro_rules! assert_slices_eq {
    ($s1: expr, $s2: expr) => {{
        let s1 = $s1;
        let s2 = $s2;
        assert!(
            testutil::are_slices_eq(s1, s2),
            "(expected) {:?} != (actual) {:?}",
            s1,
            s2,
        );
    }};
}

pub(crate) use assert_slices_eq;
