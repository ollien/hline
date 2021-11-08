#![cfg(test)]
#![allow(dead_code)]
pub(crate) mod mock_print;

pub(crate) fn are_slices_eq<T: PartialEq>(v1: &[T], v2: &[T]) -> bool {
    if v1.len() != v2.len() {
        return false;
    }

    // https://stackoverflow.com/a/29504547
    let len = v1.len();
    v1.iter().zip(v2).filter(|&(a, b)| a == b).count() == len
}
