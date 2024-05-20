use log::debug;

use crate::mm::{addr::{PPN, VA}, layout::PteFlags, map::{try_recycle, PageTable, Pte}, page::{page_alloc, page_dealloc, Page}};


pub fn alloc_test() {
    let mut pages = [PPN(0); 4];
    for ppn in pages.iter_mut() {
        *ppn = page_alloc(true).expect("Failed to allocate a page.").ppn();
    }
    assert!(pages[0] != pages[1]);
    assert!(pages[1] != pages[2]);
    assert!(pages[2] != pages[3]);

    let raw_addr = pages[1].kaddr().0 as *mut u8;
    unsafe {
        *raw_addr = 0x12;
        assert_eq!(*raw_addr, 0x12);
    }
    page_dealloc(Page::from(pages[1]));
    assert_eq!(unsafe { *raw_addr }, 0x12);
    let new_page = page_alloc(true).expect("Failed to allocate a page.").ppn();
    assert_eq!(new_page, pages[1]);
    assert_eq!(unsafe { *raw_addr }, 0); // The page should be cleared

    page_dealloc(Page::from(pages[0]));
    page_dealloc(Page::from(new_page));
    page_dealloc(Page::from(pages[2]));
    page_dealloc(Page::from(pages[3]));
    debug!("Page allocation test passed!");
}

impl PageTable {
    // TODO: Find a better to deal with this
    #[allow(clippy::mut_from_ref)]
    unsafe fn nth(&self, n: usize) -> &mut Pte {
        assert!(n < 1024);
        let base_ptr = self.page.kaddr().as_mut_ptr::<Pte>();
        &mut *base_ptr.add(n)
    }   
}

pub fn mapping_test() {
    let mut pages = [Page::new(PPN(0)); 3];
    let (pd, pd_page) = PageTable::init().unwrap();
    assert!(pd_page.ref_count() == 1);
    pages.iter_mut().for_each(|page| {
        *page = page_alloc(true).expect("Failed to allocate a page.");
    });

    // Test inserting into pd
    assert!(pd.insert(0, pages[0], VA(0x0), PteFlags::empty()).is_ok());
    assert!(pages[0].ref_count() == 1);
    let pde = unsafe { pd.nth(0) };
    assert!(pde.flags().contains(PteFlags::V) && pde.flags().contains(PteFlags::Cacheable));
    let pte = pd.lookup(VA(0x0)).unwrap().0;
    assert!(pte.flags().contains(PteFlags::V) && pte.flags().contains(PteFlags::Cacheable));
    assert_eq!(pd.va2pa(VA(0x0)).unwrap(), pages[0].into());

    // Inserting ppns[1] into 0x1000
    assert!(pd.insert(0, pages[1], VA(0x1000), PteFlags::empty()).is_ok());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), pages[1].into());
    assert!(pages[1].ref_count() == 1);

    // Replacing ppns[1] with ppns[2], ppns[1] should be deallocated
    assert!(pd.insert(0, pages[2], VA(0x1000), PteFlags::empty()).is_ok());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), pages[2].into());
    assert!(pages[1].ref_count() == 0);
    assert!(pages[2].ref_count() == 1);

    // Replacing ppns[2] with ppns[0], ppns[2] should be deallocated
    assert!(pd.insert(0, pages[0], VA(0x1000), PteFlags::empty()).is_ok());
    assert_eq!(pd.va2pa(VA(0x0)).unwrap(), pages[0].into());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), pages[0].into());
    assert!(pages[0].ref_count() == 2);
    assert!(pages[2].ref_count() == 0);

    // Check if dealloc works
    let page2 = page_alloc(true).unwrap();
    let page1 = page_alloc(true).unwrap();
    page_dealloc(page1);
    page_dealloc(page2);

    // Test removing
    // Remove ppns[0] at 0x0, it should remain at 0x1000
    pd.remove(0, VA(0x0));
    assert!(pd.va2pa(VA(0x0)).is_none());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), pages[0].into());
    assert!(pages[0].ref_count() == 1);

    // Remove ppns[0] at 0x1000, it should be deallocated
    pd.remove(0, VA(0x1000));
    assert!(pd.va2pa(VA(0x1000)).is_none());
    assert!(pages[0].ref_count() == 0);
    let page0 = page_alloc(true).unwrap();
    page_dealloc(page0);

    // Free resources
    try_recycle(pde.ppn().into());
    try_recycle(pd_page);
    debug!("Mapping test passed!");
}
