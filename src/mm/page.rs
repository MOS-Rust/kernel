#![allow(dead_code)] // TODO: Remove this

use alloc::vec::Vec;
use core::ptr::{addr_of_mut, write_bytes};

use crate::{
    mm::addr::VA,
    println,
};

use super::{
    addr::PPN,
    get_pagenum,
    layout::PAGE_SIZE,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Page {
    ppn: PPN,
    ref_count: u16,
}

impl Page {
    fn new(ppn: PPN) -> Page {
        Page { ppn, ref_count: 0 }
    }

    pub fn ppn(self) -> PPN {
        self.ppn
    }

    pub fn ref_count(self) -> u16 {
        self.ref_count
    }

    fn inc_ref(&mut self) {
        self.ref_count += 1;
    }

    fn dec_ref(&mut self) {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
    }
}

impl From<Page> for PPN {
    fn from(page: Page) -> Self {
        PPN(page.ppn.0)
    }
}

pub struct PageAllocator {
    page_table: Vec<Page>,
    free_list: Vec<PPN>,
}

impl PageAllocator {
    const fn new() -> PageAllocator {
        PageAllocator {
            page_table: Vec::new(),
            free_list: Vec::new(),
        }
    }

    fn init(&mut self, current: PPN, end: PPN) {
        self.page_table = Vec::with_capacity(get_pagenum());
        self.free_list = Vec::with_capacity(get_pagenum());
        for i in 0..current.0 {
            let mut page = Page::new(PPN(i));
            page.inc_ref();
            self.page_table.push(page);
        }
        for i in current.0..end.0 {
            self.page_table.push(Page::new(PPN(i)));
        }
        // We may want the lower addresses to be allocated earlier
        for i in (current.0..end.0).rev() {
            self.free_list.push(PPN(i));
        }
    }

    fn alloc(&mut self, clear: bool) -> Option<PPN> {
        if let Some(ppn) = self.free_list.pop() {
            if clear {
                clear_page(ppn);
            }
            Some(ppn)
        } else {
            None
        }
    }

    fn dealloc(&mut self, ppn: PPN) {
        assert!(self.page_table[ppn.0].ref_count == 0);
        self.free_list.push(ppn);
    }

    fn find_page(&self, ppn: PPN) -> Option<&Page> {
        if ppn.0 < self.page_table.len() {
            Some(&self.page_table[ppn.0])
        } else {
            None
        }
    }

    // pub fn alloc_n(&mut self, clear: bool, n: usize) -> Option<Vec<PPN>> {
    //     if self.free_list.len() < n {
    //         None
    //     } else {
    //         let mut ppns = Vec::new();
    //         for _ in 0..n {
    //             let ppn = self.free_list.pop().unwrap();
    //             if clear {
    //                 clear_page(ppn);
    //             }
    //             ppns.push(ppn);
    //         }
    //         Some(ppns)
    //     }
    // }

}

fn clear_page(ppn: PPN) {
    let va = ppn.kaddr();
    unsafe {
        write_bytes(va.as_mut_ptr::<u8>(), 0, PAGE_SIZE);
    }
}

pub static mut PAGE_ALLOCATOR: PageAllocator = PageAllocator::new();

pub fn init() {
    extern "C" {
        static mut __end_kernel: u8;
    }
    let current = PPN::from(VA(unsafe { addr_of_mut!(__end_kernel) as usize }).paddr());
    let end = PPN(get_pagenum());
    unsafe { PAGE_ALLOCATOR.init(current, end) }
}

pub fn alloc(clear: bool) -> Option<PPN> {
    unsafe { PAGE_ALLOCATOR.alloc(clear) }
}

pub fn dealloc(ppn: PPN) {
    unsafe { PAGE_ALLOCATOR.dealloc(ppn) }
}

pub fn find_page(ppn: PPN) -> Option<&'static Page> {
    unsafe { PAGE_ALLOCATOR.find_page(ppn) }
}

pub fn inc_ref(ppn: PPN) {
    unsafe { PAGE_ALLOCATOR.page_table[ppn.0].inc_ref() }
}

pub fn dec_ref(ppn: PPN) {
    unsafe { PAGE_ALLOCATOR.page_table[ppn.0].dec_ref() }
}

pub fn alloc_test() {
    let mut pages = [PPN(0); 4];
    for ppn in pages.iter_mut() {
        *ppn = alloc(true).expect("Failed to allocate a page.");
    }
    assert!(pages[0] != pages[1]);
    assert!(pages[1] != pages[2]);
    assert!(pages[2] != pages[3]);

    let raw_addr = pages[1].kaddr().0 as *mut u8;
    unsafe {
        *raw_addr = 0x12;
        assert_eq!(*raw_addr, 0x12);
    }
    dealloc(pages[1]);
    assert_eq!(unsafe { *raw_addr }, 0x12);
    let new_page = alloc(true).expect("Failed to allocate a page.");
    assert_eq!(new_page, pages[1]);
    assert_eq!(unsafe { *raw_addr }, 0); // The page should be cleared

    println!("Page allocation test passed!");
}
