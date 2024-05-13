#![allow(dead_code)]

use super::{heap, map};

pub enum TestName {
    HeapTest,
    AllocTest,
    MappingTest,
}

pub fn dispatcher(test_name: TestName) {
    match test_name {
        TestName::HeapTest => {
            heap::heap_test();
        }
        TestName::AllocTest => {
            map::alloc_test();
        }
        TestName::MappingTest => {
            map::mapping_test();
        }
    }
}