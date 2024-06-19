//! Page structure and `PageAllocator` for memory management
use crate::mutex::{FakeLock, Mutex};

use super::{
    addr::{PA, PPN, VA},
    get_pagenum,
    layout::PAGE_SIZE,
};
use alloc::vec::Vec;
use core::{
    mem::size_of,
    ptr::{addr_of_mut, write_bytes},
};
use lazy_static::lazy_static;
use log::trace;

// log_2 (512M / PAGE_SIZE) = 17
const ORDER: usize = 32;

/// Page structure for paging memory management
/// 
/// Each page is tracked by its ppn,
/// actual data is stored in page allocator
/// 
/// This structure simply wraps ppn
#[derive(Clone, Copy, Debug)]
pub struct Page {
    ppn: PPN,
}

impl Page {
    /// Construct a Page from ppn
    pub const fn new(ppn: PPN) -> Self {
        Self { ppn }
    }

    /// Acquire page's ppn
    pub const fn ppn(self) -> PPN {
        self.ppn
    }

    /// Acquire page's `ref_count`
    pub fn ref_count(self) -> u16 {
        PAGE_ALLOCATOR.lock().tracker.ref_count(self.ppn).unwrap()
    }

    /// Acquire page's kernal virtual address
    pub fn kaddr(self) -> VA {
        self.ppn.kaddr()
    }
}

impl From<PPN> for Page {
    fn from(value: PPN) -> Self {
        Self { ppn: value }
    }
}

impl From<Page> for PA {
    fn from(value: Page) -> Self {
        value.ppn().into()
    }
}

impl From<PA> for Page {
    fn from(value: PA) -> Self {
        Self::new(value.into())
    }
}

/// Struct use to track reference count of each page
#[derive(Debug)]
pub struct PageTracker {
    ppn: PPN,
    size: usize,
    page_count: usize,
}

impl PageTracker {
    /// Create a new page tracker with ppn set to PPN(0)
    const fn new() -> Self {
        Self {
            ppn: PPN(0),
            size: 0,
            page_count: 0,
        }
    }

    /// Acquire page's reference count
    pub fn ref_count(&self, ppn: PPN) -> Option<u16> {
        if ppn.0 < self.size {
            unsafe {
                let ptr = self.ppn.kaddr().as_ptr::<PageRc>().add(ppn.0);
                // trace!(
                //     "PageTracker::ref_count: ppn = {:?}, ref_count = {}",
                //     ppn,
                //     (*ptr).ref_count()
                // );
                Some((*ptr).ref_count())
            }
        } else {
            None
        }
    }

    /// Increase reference count of page at ppn
    fn inc_ref(&mut self, ppn: PPN) {
        // trace!("PageTracker::inc_ref: ppn = {:?}", ppn);
        assert!(ppn.0 < self.size);
        unsafe {
            let ptr = self.ppn.kaddr().as_mut_ptr::<PageRc>().add(ppn.0);
            (*ptr).inc_ref();
        }
    }

    /// Decrease reference count of page at ppn
    fn dec_ref(&mut self, ppn: PPN) {
        // trace!("PageTracker::dec_ref: ppn = {:?}", ppn);
        assert!(ppn.0 < self.size);
        unsafe {
            let ptr = self.ppn.kaddr().as_mut_ptr::<PageRc>().add(ppn.0);
            (*ptr).dec_ref();
        }
    }
}

#[repr(C)]
#[derive(Debug)]
/// Tracker storing page's `ref_count`
pub struct PageRc {
    __placeholder: [usize; 2],
    ref_count: u16,
}

impl PageRc {
    /// Construct a tracker with no reference
    const fn new() -> Self {
        Self {
            __placeholder: [0; 2],
            ref_count: 0,
        }
    }

    /// Acquire this tracker's `ref_count`
    const fn ref_count(&self) -> u16 {
        self.ref_count
    }

    /// Increase tracker's `ref_count`
    fn inc_ref(&mut self) {
        self.ref_count += 1;
    }

    /// Decrease tracker's `ref_count`
    /// This function will NOT decrease `ref_count`
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
    tracker: PageTracker,
    free_list: [Vec<PPN>; ORDER],
}

impl PageAllocator {
    /// Construct a new `PageAllocator`
    /// with empty pages and `free_list`
    const fn new() -> Self {
        const NEW_VEC: Vec<PPN> = Vec::new();
        Self {
            tracker: PageTracker::new(),
            free_list: [NEW_VEC; ORDER],
        }
    }

    /// Initializes the memory management system with the given range of physical page numbers.
    ///
    /// # Arguments
    ///
    /// * `current` - The starting physical page number (PPN) of the range.
    /// * `end` - The ending physical page number (PPN) of the range (exclusive).
    ///
    /// # Note
    ///
    /// PPNs in the range [0, current) are considered used and their `ref_count` is set to 1.
    ///
    /// PPNS in the range [current, end) are considered free and are added to the free list.
    ///
    /// The free list is organized as an array of vectors, where the ith vector contains all free blocks of size 2^i pages.
    fn init(&mut self, start: PPN, end: PPN) {
        let mut current = start;
        while current < end {
            let lowbit = 1 << current.0.trailing_zeros();
            let size = lowbit.min(prev_power_of_2(end - current));
            let order = size.trailing_zeros() as usize;
            self.free_list[order].push(current);
            current = current + size;
        }
        self.init_tracker(start, end);
    }

    fn init_tracker(&mut self, start: PPN, end: PPN) {
        const RC_PER_PAGE: usize = PAGE_SIZE / size_of::<PageRc>();
        let actual_size = (end.0 + RC_PER_PAGE - 1) / RC_PER_PAGE;
        let alloc_count = actual_size.next_power_of_two();
        trace!(
            "PageAllocator::init_tracker: current = {:?}, end = {:?}, alloc_count = {}, actual_size = {}",
            start,
            end,
            alloc_count,
            ((end.0 + RC_PER_PAGE - 1) / RC_PER_PAGE)
        );
        if let Some(ppn) = self.alloc(true, alloc_count) {
            self.tracker.ppn = ppn;
            self.tracker.size = end.0;
            self.tracker.page_count = actual_size;
            for i in 0..start.0 {
                unsafe {
                    let ptr = self.tracker.ppn.kaddr().as_mut_ptr::<PageRc>().add(i);
                    *ptr = PageRc::new();
                    (*ptr).inc_ref();
                }
            }
            for i in start.0..end.0 {
                unsafe {
                    let ptr = self.tracker.ppn.kaddr().as_mut_ptr::<PageRc>().add(i);
                    *ptr = PageRc::new();
                }
            }
            for i in ppn.0..actual_size {
                unsafe {
                    let ptr = self.tracker.ppn.kaddr().as_mut_ptr::<PageRc>().add(i);
                    (*ptr).inc_ref();
                }
            }
            for i in actual_size..alloc_count {
                self.dealloc(self.tracker.ppn + i, 1);
            }
        } else {
            panic!("PageTracker::init: failed to allocate pages for PageRc");
        }
    }

    /// Allocate a contiguous block of physical pages.
    ///
    /// # Arguments
    ///
    /// * `clear` - Whether to clear the allocated pages.
    /// * `size` - The number of pages to allocate, it will be rounded up to the nearest power of 2.
    ///
    /// # Returns
    ///
    /// * `Some(PPN)` - The starting physical page number (PPN) of the allocated block.
    /// * `None` - If allocation fails.
    fn alloc(&mut self, clear: bool, size: usize) -> Option<PPN> {
        let size = size.next_power_of_two();
        let order = size.trailing_zeros() as usize;
        for i in order..ORDER {
            if !self.free_list[i].is_empty() {
                for j in ((order + 1)..=i).rev() {
                    if let Some(ppn) = self.free_list[j].pop() {
                        self.free_list[j - 1].push(ppn + (1 << (j - 1)));
                        self.free_list[j - 1].push(ppn);
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
        None
    }

    /// Deallocate a previously allocated block of physical pages.
    ///
    /// # Arguments
    ///
    /// * `ppn` - The starting physical page number (PPN) of the block to deallocate.
    /// * `size` - The number of pages in the block, it will be rounded up to the nearest power of 2.
    fn dealloc(&mut self, ppn: PPN, size: usize) {
        assert!(size.is_power_of_two());
        let order = size.trailing_zeros() as usize;
        self.free_list[order].push(ppn);
        let mut ppn = ppn;
        let mut order = order;
        while order < ORDER - 1 {
            let mut flag = false;
            let buddy = ppn.0 ^ (1 << order);
            for block in &self.free_list[order] {
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

    /// Get page tracker's ppn and page count
    ///
    /// # Returns
    ///
    /// (tracker.ppn, tracker.page_count)
    pub const fn get_tracker_info(&self) -> (PPN, usize) {
        (self.tracker.ppn, self.tracker.page_count)
    }
}

/// Write 0 to ppn's page
fn clear_page(ppn: PPN) {
    let va = ppn.kaddr();
    unsafe {
        write_bytes(va.as_mut_ptr::<u8>(), 0, PAGE_SIZE);
    }
}

lazy_static! {
    /// Page allocator instance for memory management
    pub static ref PAGE_ALLOCATOR: FakeLock<PageAllocator> = FakeLock::new(PageAllocator::new());
}

/// Detect used and unused memory limit
/// Init page allocator
pub fn init() {
    extern "C" {
        static mut __end_kernel: u8;
    }
    let start = PPN::from(VA(unsafe { addr_of_mut!(__end_kernel) as usize }).paddr());
    let end = PPN(get_pagenum());
    PAGE_ALLOCATOR.lock().init(start, end)
}

/// You should use `page_alloc` instead
/// Utility function, alloc a page and return its ppn,
/// return None if there's no free page,
/// clear page if argument clear is set
#[inline]
fn alloc(clear: bool, size: usize) -> Option<PPN> {
    PAGE_ALLOCATOR.lock().alloc(clear, size)
}

/// Utility function, alloc a page and return it,
/// return None if there's no free page,
/// clear page if argument clear is set
#[inline]
pub fn page_alloc(clear: bool) -> Option<Page> {
    alloc(clear, 1).map(Page::new)
}

/// Contiguously allocate pages
#[allow(dead_code)]
#[inline]
pub fn page_alloc_contiguous(clear: bool, size: usize) -> Option<Page> {
    alloc(clear, size).map(Page::new)
}

/// You should use `page_dealloc` instead
/// Utility function, dealloc a page by its ppn,
/// panic if its `ref_count` is not 0
#[inline]
fn dealloc(ppn: PPN, size: usize) {
    PAGE_ALLOCATOR.lock().dealloc(ppn, size)
}

/// Utility function, dealloc a page,
/// panic if its `ref_count` is not 0
#[inline]
pub fn page_dealloc(page: Page) {
    dealloc(page.ppn(), 1);
}

/// Utility function, dealloc contiguous page of parameter size from page
#[allow(dead_code)]
#[inline]
pub fn page_dealloc_contiguous(page: Page, size: usize) {
    dealloc(page.ppn(), size);
}

/// Increase page's `ref_count`
#[inline]
pub fn page_inc_ref(page: Page) {
    PAGE_ALLOCATOR.lock().tracker.inc_ref(page.ppn())
}

/// Decrease page's `ref_count`
#[inline]
pub fn page_dec_ref(page: Page) {
    PAGE_ALLOCATOR.lock().tracker.dec_ref(page.ppn())
}

/// Decrease the `ref_count` of page
/// if page's `ref_count` is set to 0, deallocate the page
pub fn try_recycle(page: Page) {
    match page.ref_count() {
        0 => {
            panic!("try_recycle: page is not referenced.");
        }
        1 => {
            page_dec_ref(page);
            page_dealloc(page);
        }
        _ => {
            page_dec_ref(page);
        }
    }
}

/// Find the previous power of 2 of x
#[inline]
const fn prev_power_of_2(x: usize) -> usize {
    1 << (usize::BITS - x.leading_zeros() - 1)
}
