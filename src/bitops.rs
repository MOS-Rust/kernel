#![allow(dead_code)]

/// Round up `a` to the nearest multiple of `n`.
#[macro_export]
macro_rules! round {
    ($a:expr, $n:expr) => {
        ($a + $n - 1) & !($n - 1)
    };
}

/// Round down `a` to the nearest multiple of `n`.
#[macro_export]
macro_rules! round_down {
    ($a:expr, $n:expr) => {
        $a & !($n - 1)
    };
}
