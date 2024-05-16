use log::debug;

use crate::mm::{addr::{PPN, VA}, layout::PteFlags, map::{PageTable, Pte}, page::{alloc, dealloc, page_alloc, page_dealloc, Page}};


pub fn alloc_test() {
    let mut pages = [PPN(0); 4];
    for ppn in pages.iter_mut() {
        *ppn = alloc(true, 1).expect("Failed to allocate a page.");
    }
    assert!(pages[0] != pages[1]);
    assert!(pages[1] != pages[2]);
    assert!(pages[2] != pages[3]);

    let raw_addr = pages[1].kaddr().0 as *mut u8;
    unsafe {
        *raw_addr = 0x12;
        assert_eq!(*raw_addr, 0x12);
    }
    dealloc(pages[1], 1);
    assert_eq!(unsafe { *raw_addr }, 0x12);
    let new_page = alloc(true, 1).expect("Failed to allocate a page.");
    assert_eq!(new_page, pages[1]);
    assert_eq!(unsafe { *raw_addr }, 0); // The page should be cleared

    dealloc(pages[0], 1);
    dealloc(new_page, 1);
    dealloc(pages[2], 1);
    dealloc(pages[3], 1);
    debug!("Page allocation test passed!");
}
impl PageTable {
    unsafe fn nth(&self, n: usize) -> &mut Pte {
        assert!(n < 1024);
        let base_ptr = self.page.ppn().kaddr().as_mut_ptr::<Pte>();
        &mut *base_ptr.add(n)
    }   
}

pub fn mapping_test() {
    let mut pages = [Page::new(PPN(0)); 3];
    let (pd, pd_page) = PageTable::init().unwrap();
    assert!(pd_page.ref_count() == 1);
    for i in 0..3 {
        pages[i] = page_alloc(true).expect("Failed to allocate a page.");
    }

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
    assert_eq!(page0, pages[0]);
    page_dealloc(page0);

    // Free resources
    PageTable::try_recycle(pde.ppn().into());
    PageTable::try_recycle(pd_page);
    debug!("Mapping test passed!");
}
