//! Page structure and PageAllocator for memory management
use alloc::vec::Vec;
use core::ptr::{addr_of_mut, write_bytes};
use crate::mm::addr::VA;

use super::{
    addr::{PA, PPN},
    get_pagenum,
    layout::PAGE_SIZE,
};

// log_2 (512M / PAGE_SIZE) = 17
const ORDER: usize = 32;

/// Page structure for paging memory management
/// Each page is tracked by its ppn,
/// actual data is stored in page allocator
/// This structure simply wraps ppn
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Page {
    ppn: PPN,
}

impl Page {
    /// Construct a Page from ppn
    pub fn new(ppn: PPN) -> Self {
        Page { ppn }
    }

    /// Acquire page's ppn
    pub fn ppn(self) -> PPN {
        self.ppn
    }

    /// Acquire page's ref_count
    pub fn ref_count(self) -> u16 {
        let tracker = find_page(self);
        tracker.unwrap().ref_count()
    }
}

impl From<PPN> for Page {
    fn from(value: PPN) -> Self {
        Page { ppn: value }
    }
}

impl From<Page> for PA {
    fn from(value: Page) -> Self {
        value.ppn().into()
    }
}

impl From<PA> for Page {
    fn from(value: PA) -> Self {
        Page::new(value.into())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Tracker storing page's ref_count
pub struct PageTracker {
    ref_count: u16,
}

impl PageTracker {
    /// Construct a tracker with no reference
    fn new() -> PageTracker {
        PageTracker { ref_count: 0 }
    }

    /// Acquire this tracker's ref_count
    pub fn ref_count(self) -> u16 {
        self.ref_count
    }

    /// Increase tracker's ref_count
    fn inc_ref(&mut self) {
        self.ref_count += 1;
    }

    /// Decrease tracker's ref_count
    /// This function will NOT decrease ref_count
    /// if it is less than 0
    fn dec_ref(&mut self) {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
    }
}

/// Structure storing actual pages and refrence count
/// Manages page allocation and deallocation
pub struct PageAllocator {
    pages: Vec<PageTracker>,
    free_list: [Vec<PPN>; ORDER],
}

impl PageAllocator {
    const fn new() -> Self {
        const NEW_VEC: Vec<PPN> = Vec::new();
        PageAllocator {
            pages: Vec::new(),
            free_list: [NEW_VEC; ORDER],
        }
    }

    fn init(&mut self, current: PPN, end: PPN) {
        const NEW_VEC: Vec<PPN> = Vec::new();
        self.pages = Vec::with_capacity(get_pagenum());
        self.free_list = [NEW_VEC; ORDER];
        for _ in 0..current.0 {
            let mut page = PageTracker::new();
            page.inc_ref();
            self.pages.push(page);
        }
        for _ in current.0..end.0 {
            self.pages.push(PageTracker::new());
        }
        let mut current = current;
        while current < end {
            let lowbit = 1 << current.0.trailing_zeros();
            let size = lowbit.min(prev_power_of_2(end - current)) as usize;
            let order = size.trailing_zeros() as usize;
            self.free_list[order].push(current);
            current = current + size;
        }
    }

    fn alloc(&mut self, clear: bool, size: usize) -> Option<PPN> {
        let size = size.next_power_of_two();
        let order = size.trailing_zeros() as usize;
        assert!(order < ORDER);
        for i in order..ORDER {
            if !self.free_list[i].is_empty() {
                for j in (order + 1..i + 1).rev() {
                    if let Some(ppn) = self.free_list[j].pop() {
                        self.free_list[j - 1].push(ppn);
                        self.free_list[j - 1].push(ppn + (1 << (j - 1)));
                    } else {
                        return None;
                    }
                }
                let ppn = self.free_list[order].pop().expect("There should be a page");
                if clear {
                    for j in 0..size {
                        clear_page(ppn + j);
                    }
                }
                return Some(ppn);
            }
        }
        return None;
    }

    fn dealloc(&mut self, ppn: PPN, size: usize) {
        let size = size.next_power_of_two();
        let order = size.trailing_zeros() as usize;
        assert!(order < ORDER);
        self.free_list[order].push(ppn);
        let mut ppn = ppn;
        let mut order = order;
        while order < ORDER - 1 {
            let mut flag = false;
            let buddy = ppn.0 ^ (1 << order);
            for block in self.free_list[order].iter() {
                if block.0 == buddy {
                    flag = true;
                    break;
                }
            }
            if flag {
                self.free_list[order].retain(|x| x.0 != buddy && *x != ppn);
                ppn = PPN(ppn.0 & buddy);
                order += 1;
                self.free_list[order].push(ppn);
            } else {
                break;
            }
        }
    }

    fn find_page(&self, ppn: PPN) -> Option<&PageTracker> {
        if ppn.0 < self.pages.len() {
            Some(&self.pages[ppn.0])
        } else {
            None
        }
    }
}

// impl PageAllocator {
//     /// Construct an empty page allocator
//     const fn new() -> PageAllocator {
//         PageAllocator {
//             pages: Vec::new(),
//             free_list: Vec::new(),
//         }
//     }
//
//     /// Init page allocator, acquire empty memory and generate empty pages
//     ///
//     /// # arguments
//     /// this part should be supplemented
//     /// * current
//     /// * end
//     ///
//     fn init(&mut self, current: PPN, end: PPN) {
//         self.pages = Vec::with_capacity(get_pagenum());
//         self.free_list = Vec::with_capacity(get_pagenum());
//         for _ in 0..current.0 {
//             let mut page = PageTracker::new();
//             page.inc_ref();
//             self.pages.push(page);
//         }
//         for _ in current.0..end.0 {
//             self.pages.push(PageTracker::new());
//         }
//         // We may want the lower addresses to be allocated earlier
//         for i in (current.0..end.0).rev() {
//             self.free_list.push(PPN(i));
//         }
//     }
//
//     /// Allocate a page and return its ppn
//     /// Clear page if arugument clear is set
//     fn alloc(&mut self, clear: bool) -> Option<PPN> {
//         if let Some(ppn) = self.free_list.pop() {
//             if clear {
//                 clear_page(ppn);
//             }
//             Some(ppn)
//         } else {
//             None
//         }
//     }
//
//     /// Deallocate a page by its ppn, insert it into page free list
//     /// Panic if page's ref_count is not 0
//     fn dealloc(&mut self, ppn: PPN) {
//         assert!(self.pages[ppn.0].ref_count == 0);
//         self.free_list.push(ppn);
//     }
//
//     /// Find a page's tracker by its ppn
//     fn find_page(&self, ppn: PPN) -> Option<&PageTracker> {
//         if ppn.0 < self.pages.len() {
//             Some(&self.pages[ppn.0])
//         } else {
//             None
//         }
//     }
// }

/// Write 0 to ppn's page
fn clear_page(ppn: PPN) {
    let va = ppn.kaddr();
    unsafe {
        write_bytes(va.as_mut_ptr::<u8>(), 0, PAGE_SIZE);
    }
}

/// Page allocator instance for memory management
pub static mut PAGE_ALLOCATOR: PageAllocator = PageAllocator::new();

/// Detect used and unused memory limit
/// Init page allocator
pub fn init() {
    extern "C" {
        static mut __end_kernel: u8;
    }
    let current = PPN::from(VA(unsafe { addr_of_mut!(__end_kernel) as usize }).paddr());
    let end = PPN(get_pagenum());
    unsafe { PAGE_ALLOCATOR.init(current, end) }
}

/// You should use page_alloc instead
/// Utility function, alloc a page and return its ppn,
/// return None if there's no free page,
/// clear page if argument clear is set
#[inline]
pub fn alloc(clear: bool, size: usize) -> Option<PPN> {
    unsafe { PAGE_ALLOCATOR.alloc(clear, size) }
}

/// Utility function, alloc a page and return it,
/// return None if there's no free page,
/// clear page if argument clear is set
#[inline]
pub fn page_alloc(clear: bool) -> Option<Page> {
    if let Some(ppn) = alloc(clear, 1) {
        Some(Page::new(ppn))
    } else {
        None
    }
}

#[inline]
#[allow(dead_code)]
pub fn page_alloc_contiguous(clear: bool, size: usize) -> Option<Page> {
    if let Some(ppn) = alloc(clear, size) {
        Some(Page::new(ppn))
    } else {
        None
    }
}

/// You should use page_dealloc instead
/// Utility function, dealloc a page by its ppn,
/// panic if its ref_count is not 0
#[inline]
pub fn dealloc(ppn: PPN, size: usize) {
    unsafe { PAGE_ALLOCATOR.dealloc(ppn, size) }
}

/// Utility function, dealloc a page,
/// panic if its ref_count is not 0
#[inline]
pub fn page_dealloc(page: Page) {
    dealloc(page.ppn(), 1)
}

#[inline]
#[allow(dead_code)]
pub fn page_dealloc_contiguous(page: Page, size: usize) {
    dealloc(page.ppn(), size)
}

/// Do not use this function to acquire page's ref_count,
/// you should use page.ref_count() instead
/// Find the page's tracker
#[inline]
pub fn find_page(page: Page) -> Option<&'static PageTracker> {
    unsafe { PAGE_ALLOCATOR.find_page(page.ppn()) }
}

/// You should use page_inc_ref instead
/// Increase page's ref_count by its ppn
#[inline]
pub fn inc_ref(ppn: PPN) {
    unsafe { PAGE_ALLOCATOR.pages[ppn.0].inc_ref() }
}

/// Increase page's ref_count
#[inline]
pub fn page_inc_ref(page: Page) {
    inc_ref(page.ppn())
}

/// You should use page_dec_ref instead
/// Decrease page's ref_count by its ppn
#[inline]
pub fn dec_ref(ppn: PPN) {
    unsafe { PAGE_ALLOCATOR.pages[ppn.0].dec_ref() }
}

/// Decrease page's ref_count
#[inline]
pub fn page_dec_ref(page: Page) {
    dec_ref(page.ppn())
}

fn prev_power_of_2(x: usize) -> usize {
    1 << (usize::BITS - x.leading_zeros() - 1)
}


