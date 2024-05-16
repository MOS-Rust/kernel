use alloc::{string::String, vec::Vec};
use log::debug;

/// Perform a heap test.
pub fn heap_test() {
    let mut v = Vec::<u8>::new();
    for i in 0..=255 {
        v.push(i);
    }
    v.iter().enumerate().take(255 + 1).for_each(|(i, &x)| {
        assert_eq!(x, i as u8);
    });
    let s = String::from("Hello, world!");
    assert_eq!(s, "Hello, world!");
    drop(v);
    debug!("Heap test passed!");
}
