//! Page structure and PageAllocator for memory management
use alloc::vec::Vec;
use core::ptr::{addr_of_mut, write_bytes};

use crate::mm::addr::VA;

use super::{
    addr::{PA, PPN},
    get_pagenum,
    layout::PAGE_SIZE,
};

/// Page structure for paging memory management
/// each page is tracked by its ppn,
/// actual data is stored in page allocator
/// this structure simply wraps ppn
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Page {
    ppn: PPN,
}

impl Page {
    /// construct a Page from ppn
    pub const fn new(ppn: PPN) -> Self {
        Page { ppn }
    }

    /// acquire page's ppn
    pub fn ppn(self) -> PPN {
        self.ppn
    }

    /// acquire page's ref_count
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
/// tracker storing page's ref_count
pub struct PageTracker {
    ref_count: u16,
}

impl PageTracker {
    /// construct a tracker with no reference
    fn new() -> PageTracker {
        PageTracker { ref_count: 0 }
    }

    /// acquire this tracker's ref_count
    pub fn ref_count(self) -> u16 {
        self.ref_count
    }

    /// increase tracker's ref_count
    fn inc_ref(&mut self) {
        self.ref_count += 1;
    }

    /// deincrease tracker's ref_count
    /// this function will NOT deincrease ref_count 
    /// if it is less than 0
    fn dec_ref(&mut self) {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
    }
}

/// structure storing actual pages and refrence count
/// manages page allocation and deallocation
pub struct PageAllocator {
    pages: Vec<PageTracker>,
    free_list: Vec<PPN>,
}

impl PageAllocator {
    /// construct an empty page allocator
    const fn new() -> PageAllocator {
        PageAllocator {
            pages: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// init page allocator, acquire empty memory and generate empty pages
    fn init(&mut self, current: PPN, end: PPN) {
        self.pages = Vec::with_capacity(get_pagenum());
        self.free_list = Vec::with_capacity(get_pagenum());
        for _ in 0..current.0 {
            let mut page = PageTracker::new();
            page.inc_ref();
            self.pages.push(page);
        }
        for _ in current.0..end.0 {
            self.pages.push(PageTracker::new());
        }
        // We may want the lower addresses to be allocated earlier
        for i in (current.0..end.0).rev() {
            self.free_list.push(PPN(i));
        }
    }

    /// allocate a page and return its ppn
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

    /// deallocator a page by its ppn, insert it into page free list
    /// panic if page's ref_count is not 0
    fn dealloc(&mut self, ppn: PPN) {
        assert!(self.pages[ppn.0].ref_count == 0);
        self.free_list.push(ppn);
    }

    /// find a page's tracker by its ppn
    fn find_page(&self, ppn: PPN) -> Option<&PageTracker> {
        if ppn.0 < self.pages.len() {
            Some(&self.pages[ppn.0])
        } else {
            None
        }
    }
}

/// write 0 to ppn's page
fn clear_page(ppn: PPN) {
    let va = ppn.kaddr();
    unsafe {
        write_bytes(va.as_mut_ptr::<u8>(), 0, PAGE_SIZE);
    }
}

/// page allocator instance for memory management
pub static mut PAGE_ALLOCATOR: PageAllocator = PageAllocator::new();

/// detect used and unused memory limit
/// init page allocator
pub fn init() {
    extern "C" {
        static mut __end_kernel: u8;
    }
    let current = PPN::from(VA(unsafe { addr_of_mut!(__end_kernel) as usize }).paddr());
    let end = PPN(get_pagenum());
    unsafe { PAGE_ALLOCATOR.init(current, end) }
}

/// you should use page_alloc instead
/// utility function, alloc a page and return its ppn
/// return None if there's no free page
#[inline]
pub fn alloc(clear: bool) -> Option<PPN> {
    unsafe { PAGE_ALLOCATOR.alloc(clear) }
}

/// utility function, alloc a page and return it
/// return None if there's no free page
#[inline]
pub fn page_alloc(clear: bool) -> Option<Page> {
    if let Some(ppn) = alloc(clear) {
        Some(Page::new(ppn))
    } else {
        None
    }
}

/// you should use page_dealloc instread
/// utility function, dealloc a page by its ppn
/// panic if its ref_count is not 0
#[inline]
pub fn dealloc(ppn: PPN) {
    unsafe { PAGE_ALLOCATOR.dealloc(ppn) }
}

/// utility function, dealloc a page
/// panic if its ref_count is not 0
#[inline]
pub fn page_dealloc(page: Page) {
    dealloc(page.ppn())
}

/// 
#[inline]
pub fn find_page(page: Page) -> Option<&'static PageTracker> {
    unsafe { PAGE_ALLOCATOR.find_page(page.ppn()) }
}

#[inline]
pub fn inc_ref(ppn: PPN) {
    unsafe { PAGE_ALLOCATOR.pages[ppn.0].inc_ref() }
}

#[inline]
pub fn page_inc_ref(page: Page) {
    inc_ref(page.ppn())
}

#[inline]
pub fn dec_ref(ppn: PPN) {
    unsafe { PAGE_ALLOCATOR.pages[ppn.0].dec_ref() }
}

#[inline]
pub fn page_dec_ref(page: Page) {
    dec_ref(page.ppn())
}
