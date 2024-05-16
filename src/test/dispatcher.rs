#![allow(dead_code)]

use super::{env, heap, map};

pub enum TestName {
    Heap,
    Alloc,
    Mapping,
    Env,
}

pub fn dispatcher(test_name: TestName) {
    match test_name {
        TestName::Heap => {
            heap::heap_test();
        }
        TestName::Alloc => {
            map::alloc_test();
        }
        TestName::Mapping => {
            map::mapping_test();
        }
        TestName::Env => {
            env::env_test();
        }
    }
}