#![allow(dead_code)]

#[inline]
pub fn round(a: usize, n: usize) -> usize {
    (a + n - 1) & !(n - 1)
}

#[inline]
pub fn round_down(a: usize, n: usize) -> usize {
    a & !(n - 1)
}
