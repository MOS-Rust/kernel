#![allow(dead_code)] // TODO: Remove this

use alloc::vec::Vec;
use core::ptr::{addr_of_mut, write_bytes};
use crate::error::MosError;

use crate::mm::addr::VA;

use super::{
    addr::{PA, PPN},
    get_pagenum,
    layout::PAGE_SIZE,
};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Page {
    pub ppn: PPN,
    pub ref_count: u16,
}

impl Page {
    pub fn new(ppn: PPN) -> Page {
        Page { ppn, ref_count: 0 }
    }

    pub fn inc_ref(&mut self) {
        self.ref_count += 1;
    }

    pub fn dec_ref(&mut self) {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
    }

    pub fn to_pa(&self) -> PA {
        PA::from(PPN::from(self.ppn.0))
    }

    pub fn to_kva(&self) -> VA {
        self.to_pa().kaddr()
    }
}

impl From<Page> for PPN {
    fn from(page: Page) -> Self {
        PPN(page.ppn.0)
    }
}

impl From<Page> for PA {
    fn from(page: Page) -> Self {
        PA::from(page.ppn.0)
    }
}

pub struct PageAllocator {
    page_table: Vec<Page>,
    free_list: Vec<PPN>,
}

impl PageAllocator {
    pub const fn new() -> PageAllocator {
        PageAllocator {
            page_table: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn init(&mut self, current: PPN, end: PPN) {
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

    pub fn alloc(&mut self, clear: bool) -> Result<PPN, MosError> {
        if let Some(ppn) = self.free_list.pop() {
            if clear {
                clear_page(ppn);
            }
            Ok(ppn)
        } else {
            Err(MosError::NoMem)
        }
    }

    pub fn dealloc(&mut self, ppn: PPN) {
        assert!(self.page_table[ppn.0].ref_count == 0);
        self.free_list.push(ppn);
    }

    pub fn find_page(&mut self, ppn: PPN) -> Option<&mut Page> {
        if ppn.0 < self.page_table.len() {
            Some(&mut self.page_table[ppn.0])
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
    let va = PA::from(ppn).kaddr();
    unsafe {
        write_bytes(va.0 as *mut u8, 0, PAGE_SIZE);  //? is this correct?
    }
}

// TODO: make this thread-safe
// TODO: find a wrapper to make alloc and dealloc safe
pub static mut PAGE_ALLOCATOR: PageAllocator = PageAllocator::new();

pub fn init() {
    extern "C" {
        static mut __end_kernel: u8;
    }
    let current = PPN::from(VA::from(unsafe { addr_of_mut!(__end_kernel) as usize }).paddr());
    let end = PPN::from(get_pagenum());
    unsafe { PAGE_ALLOCATOR.init(current, end) }
}

pub fn inc_ref(ppn: PPN) {
    unsafe { PAGE_ALLOCATOR.page_table[ppn.0].inc_ref() }
}

pub fn dec_ref(ppn: PPN) {
    unsafe { PAGE_ALLOCATOR.page_table[ppn.0].dec_ref() }
}

pub fn alloc_test() {
    // TODO: return type of page alloc is modified, rewrite test
    // let ppn = unsafe { PAGE_ALLOCATOR.alloc(true).unwrap() };
    // println!("Allocated page: {:?}", ppn);
    // unsafe { PAGE_ALLOCATOR.dealloc(ppn) };
    // println!("Deallocated page: {:?}", ppn);
    // // TODO: populating test
}
